//! Snapshot and restore sessions from serialized state.

use std::collections::BTreeMap;

use anyhow::{ensure, Context, Result};
use prost::Message;
use sshx_core::{
    proto::{SerializedSession, SerializedShell},
    Sid, Uid,
};

use super::{Metadata, Session, State};
use crate::web::protocol::WsWinsize;

/// Persist at most this many bytes of output in storage, per shell.
const SHELL_SNAPSHOT_BYTES: u64 = 1 << 15; // 32 KiB

const MAX_SNAPSHOT_SIZE: usize = 1 << 22; // 4 MiB

impl Session {
    /// Snapshot the session, returning a compressed representation.
    pub fn snapshot(&self) -> Result<Vec<u8>> {
        let ids = self.counter.get_current_values();
        let winsizes: BTreeMap<Sid, WsWinsize> = self.source.borrow().iter().cloned().collect();
        let message = SerializedSession {
            encrypted_zeros: self.metadata().encrypted_zeros.clone(),
            shells: self
                .shells
                .read()
                .iter()
                .map(|(sid, shell)| {
                    // Prune off data until its total length is at most `SHELL_SNAPSHOT_BYTES`.
                    let mut prefix = 0;
                    let mut chunk_offset = shell.chunk_offset;
                    let mut byte_offset = shell.byte_offset;

                    for i in 0..shell.data.len() {
                        if shell.seqnum - byte_offset > SHELL_SNAPSHOT_BYTES {
                            prefix += 1;
                            chunk_offset += 1;
                            byte_offset += shell.data[i].len() as u64;
                        } else {
                            break;
                        }
                    }

                    let winsize = winsizes.get(sid).cloned().unwrap_or_default();
                    let shell = SerializedShell {
                        seqnum: shell.seqnum,
                        data: shell.data[prefix..].to_vec(),
                        chunk_offset,
                        byte_offset,
                        closed: shell.closed,
                        winsize_x: winsize.x,
                        winsize_y: winsize.y,
                        winsize_rows: winsize.rows.into(),
                        winsize_cols: winsize.cols.into(),
                    };
                    (sid.0, shell)
                })
                .collect(),
            next_sid: ids.0 .0,
            next_uid: ids.1 .0,
            name: self.metadata().name.clone(),
            write_password_hash: self.metadata().write_password_hash.clone(),
        };
        let data = message.encode_to_vec();
        ensure!(data.len() < MAX_SNAPSHOT_SIZE, "snapshot too large");
        Ok(zstd::bulk::compress(&data, 3)?)
    }

    /// Restore the session from a previous compressed snapshot.
    pub fn restore(data: &[u8]) -> Result<Self> {
        let data = zstd::bulk::decompress(data, MAX_SNAPSHOT_SIZE)?;
        let message = SerializedSession::decode(&*data)?;

        let metadata = Metadata {
            encrypted_zeros: message.encrypted_zeros,
            name: message.name,
            write_password_hash: message.write_password_hash,
        };

        let session = Self::new(metadata);
        let mut shells = session.shells.write();
        let mut winsizes = Vec::new();
        for (sid, shell) in message.shells {
            winsizes.push((
                Sid(sid),
                WsWinsize {
                    x: shell.winsize_x,
                    y: shell.winsize_y,
                    rows: shell.winsize_rows.try_into().context("rows overflow")?,
                    cols: shell.winsize_cols.try_into().context("cols overflow")?,
                },
            ));
            let shell = State {
                seqnum: shell.seqnum,
                data: shell.data,
                chunk_offset: shell.chunk_offset,
                byte_offset: shell.byte_offset,
                closed: shell.closed,
                notify: Default::default(),
            };
            shells.insert(Sid(sid), shell);
        }
        drop(shells);
        session.source.send_replace(winsizes);
        session
            .counter
            .set_current_values(Sid(message.next_sid), Uid(message.next_uid));

        Ok(session)
    }
}

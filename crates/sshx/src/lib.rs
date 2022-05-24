//! Library code for the sshx command-line client application.
//!
//! This crate does not forbid use of unsafe code because it needs to interact
//! with operating-system APIs to access pseudoterminal (PTY) devices.

#![warn(missing_docs)]

pub mod terminal;

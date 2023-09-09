//! Library code for the sshx command-line client application.
//!
//! This crate does not forbid use of unsafe code because it needs to interact
//! with operating-system APIs to access pseudoterminal (PTY) devices.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod controller;
pub mod encrypt;
pub mod runner;
pub mod terminal;

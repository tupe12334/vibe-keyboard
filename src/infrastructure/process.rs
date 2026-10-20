//! Runs a program with a time limit. The spawn lives in `process/os.rs`, which the coverage gate
//! skips. Shared by the Claude sessions and the focused session code.

mod os;

pub use os::run;

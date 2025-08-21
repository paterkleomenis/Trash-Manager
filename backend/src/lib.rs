//! UI-agnostic process management library for Linux.
//!
//! Provides functions for listing processes, killing processes, killing process trees, and killing cgroups.
//! Uses `nix` and `procfs` for system interaction.

mod process_kill;
mod process_list;
mod types;

pub use process_kill::{kill_cgroup, kill_pid, kill_tree};
pub use process_list::list_processes;
pub use types::{ProcError, ProcessInfo};

//! Data types and error definitions for process management.

use thiserror::Error;

/// Represents a process entry.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub state: String,
    pub ppid: i32,
}

/// Errors that can occur during process management.
#[derive(Error, Debug)]
pub enum ProcError {
    #[error("Permission denied for PID {0}")]
    PermissionDenied(i32),
    #[error("Process {0} is in unkillable state (D or zombie)")]
    UnkillableState(i32),
    #[error("Process {0} not found")]
    NotFound(i32),
    #[error("Failed to send signal to PID {0}: {1}")]
    SignalError(i32, String),
    #[error("Cgroup operation failed: {0}")]
    CgroupError(String),
    #[error("Other error: {0}")]
    Other(String),
    #[error("Procfs error: {0}")]
    ProcfsError(String),
}

impl From<procfs::ProcError> for ProcError {
    fn from(err: procfs::ProcError) -> Self {
        ProcError::ProcfsError(err.to_string())
    }
}

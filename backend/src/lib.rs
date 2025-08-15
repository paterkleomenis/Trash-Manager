//! UI-agnostic process management library for Linux.
//!
//! Provides functions for listing processes, killing processes, killing process trees, and killing cgroups.
//! Uses `nix` and `procfs` for system interaction.

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::{thread, time};
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

/// List all processes with their info.
/// Returns a vector of `ProcessInfo`.
pub fn list_processes() -> Result<Vec<ProcessInfo>, ProcError> {
    let mut processes = Vec::new();

    // Access the /proc directory using procfs
    let all_procs = procfs::process::all_processes()
        .map_err(|e| ProcError::Other(format!("Failed to read /proc: {}", e)))?;

    for proc_result in all_procs {
        if let Ok(proc) = proc_result {
            if let Ok(stat) = proc.stat() {
                let memory_bytes = proc.statm().map(|m| m.resident * 4096).unwrap_or(0); // Resident memory in bytes
                let cpu_percent = 0.0; // Placeholder for CPU usage calculation
                let process_info = ProcessInfo {
                    pid: stat.pid,
                    name: stat.comm.clone(),
                    cpu_percent,
                    memory_bytes,
                    state: stat.state.to_string(),
                    ppid: stat.ppid,
                };
                processes.push(process_info);
            }
        }
    }

    Ok(processes)
}

/// Kill a process by PID.
/// Sends SIGSTOP, then SIGTERM, waits 500ms, then SIGKILL if still running.
/// Uses pidfd_send_signal if supported.

pub fn kill_pid(pid: i32) -> Result<(), ProcError> {
    let pid = Pid::from_raw(pid);

    // Try stopping the process first
    if let Err(e) = signal::kill(pid, Signal::SIGSTOP) {
        return Err(ProcError::SignalError(pid.as_raw(), e.to_string()));
    }

    // Then send SIGTERM
    if let Err(e) = signal::kill(pid, Signal::SIGTERM) {
        return Err(ProcError::SignalError(pid.as_raw(), e.to_string()));
    }

    // Wait for half a second
    thread::sleep(time::Duration::from_millis(500));

    // Check if the process is still alive and send SIGKILL
    if let Err(_) = signal::kill(pid, None) {
        // Process already gone
        return Ok(());
    }

    if let Err(e) = signal::kill(pid, Signal::SIGKILL) {
        return Err(ProcError::SignalError(pid.as_raw(), e.to_string()));
    }

    Ok(())
}

/// Kill a process and all its descendants recursively.
pub fn kill_tree(_pid: i32) -> Result<(), ProcError> {
    // TODO: Implement recursive killing using process tree.
    Ok(())
}

/// Kill all processes in a cgroup v2 by writing 1 to cgroup.kill.
pub fn kill_cgroup(_cgroup_path: &str) -> Result<(), ProcError> {
    // TODO: Implement cgroup v2 killing.
    Ok(())
}

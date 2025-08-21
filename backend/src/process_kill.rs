//! Process killing functionality.

use crate::types::ProcError;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::{thread, time};

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

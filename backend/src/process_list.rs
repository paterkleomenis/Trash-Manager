//! Process listing functionality with real CPU calculation.

use crate::types::{ProcError, ProcessInfo};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

// Global CPU tracking state
static CPU_TRACKER: Mutex<Option<CpuTracker>> = Mutex::new(None);

#[allow(dead_code)]
struct ProcessCpuData {
    utime: u64,
    stime: u64,
    total_time: u64,
    last_update: Instant,
}

struct CpuTracker {
    process_data: HashMap<i32, ProcessCpuData>,
    last_system_total: u64,
    last_system_idle: u64,
    last_system_update: Instant,
    clock_ticks_per_second: u64,
}

impl CpuTracker {
    fn new() -> Self {
        let clock_ticks_per_second = 100; // Standard Linux clock ticks per second
        let (system_total, system_idle) = Self::read_system_cpu_times();

        Self {
            process_data: HashMap::new(),
            last_system_total: system_total,
            last_system_idle: system_idle,
            last_system_update: Instant::now(),
            clock_ticks_per_second,
        }
    }

    fn read_system_cpu_times() -> (u64, u64) {
        // Read from /proc/stat to get system-wide CPU times
        if let Ok(stat_content) = std::fs::read_to_string("/proc/stat") {
            if let Some(cpu_line) = stat_content.lines().next() {
                if cpu_line.starts_with("cpu ") {
                    let values: Vec<u64> = cpu_line
                        .split_whitespace()
                        .skip(1) // Skip "cpu"
                        .take(8) // user, nice, system, idle, iowait, irq, softirq, steal
                        .filter_map(|s| s.parse().ok())
                        .collect();

                    if values.len() >= 4 {
                        let total = values.iter().sum();
                        let idle = values[3]; // idle time
                        return (total, idle);
                    }
                }
            }
        }
        (0, 0)
    }

    fn calculate_cpu_percent(&mut self, pid: i32, utime: u64, stime: u64) -> f32 {
        let now = Instant::now();
        let total_time = utime + stime;

        // Update system CPU times
        let (current_system_total, current_system_idle) = Self::read_system_cpu_times();

        if let Some(prev_data) = self.process_data.get(&pid) {
            let elapsed_seconds = now.duration_since(prev_data.last_update).as_secs_f64();
            let system_elapsed_seconds = now.duration_since(self.last_system_update).as_secs_f64();

            // Need at least 1 second for meaningful measurement
            if elapsed_seconds >= 1.0 && system_elapsed_seconds >= 1.0 {
                // Calculate process CPU time difference
                let process_time_diff = total_time.saturating_sub(prev_data.total_time);
                let process_cpu_seconds =
                    process_time_diff as f64 / self.clock_ticks_per_second as f64;

                // Calculate system CPU time difference
                let system_total_diff = current_system_total.saturating_sub(self.last_system_total);
                let system_cpu_seconds =
                    system_total_diff as f64 / self.clock_ticks_per_second as f64;

                // CPU percentage = (process CPU time / system CPU time) * 100
                let cpu_percent = if system_cpu_seconds > 0.0 {
                    (process_cpu_seconds / system_cpu_seconds) * 100.0
                } else {
                    // Fallback: simple time-based calculation
                    (process_cpu_seconds / elapsed_seconds) * 100.0
                };

                // Update stored data
                self.process_data.insert(
                    pid,
                    ProcessCpuData {
                        utime,
                        stime,
                        total_time,
                        last_update: now,
                    },
                );

                // Update system times if this is a fresh measurement
                if system_elapsed_seconds >= 1.0 {
                    self.last_system_total = current_system_total;
                    self.last_system_idle = current_system_idle;
                    self.last_system_update = now;
                }

                // Return reasonable CPU percentage
                (cpu_percent.min(100.0).max(0.0)) as f32
            } else {
                // Not enough time elapsed, return 0
                0.0
            }
        } else {
            // First time seeing this process, store data but return 0
            self.process_data.insert(
                pid,
                ProcessCpuData {
                    utime,
                    stime,
                    total_time,
                    last_update: now,
                },
            );
            0.0
        }
    }

    fn cleanup_old_processes(&mut self, current_pids: &[i32]) {
        let current_pids_set: std::collections::HashSet<i32> =
            current_pids.iter().cloned().collect();
        self.process_data
            .retain(|pid, _| current_pids_set.contains(pid));
    }
}

/// List all processes with their info including real CPU usage.
/// Returns a vector of `ProcessInfo`.
pub fn list_processes() -> Result<Vec<ProcessInfo>, ProcError> {
    let mut processes = Vec::new();

    // Access the /proc directory using procfs
    let all_procs = procfs::process::all_processes()
        .map_err(|e| ProcError::Other(format!("Failed to read /proc: {}", e)))?;

    // Initialize CPU tracker if needed
    {
        let mut tracker_guard = CPU_TRACKER.lock().unwrap();
        if tracker_guard.is_none() {
            *tracker_guard = Some(CpuTracker::new());
        }
    }

    let mut current_pids = Vec::new();

    for proc_result in all_procs {
        if let Ok(proc) = proc_result {
            if let Ok(stat) = proc.stat() {
                current_pids.push(stat.pid);

                let memory_bytes = proc.statm().map(|m| m.resident * 4096).unwrap_or(0);

                // Calculate real CPU percentage
                let cpu_percent = {
                    let mut tracker_guard = CPU_TRACKER.lock().unwrap();
                    if let Some(ref mut tracker) = tracker_guard.as_mut() {
                        tracker.calculate_cpu_percent(stat.pid, stat.utime, stat.stime)
                    } else {
                        0.0
                    }
                };

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

    // Clean up old process data
    {
        let mut tracker_guard = CPU_TRACKER.lock().unwrap();
        if let Some(ref mut tracker) = tracker_guard.as_mut() {
            tracker.cleanup_old_processes(&current_pids);
        }
    }

    Ok(processes)
}

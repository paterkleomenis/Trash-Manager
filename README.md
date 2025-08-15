# Trash Manager

Trash Manager is a modular Linux process management tool with a live-updating egui-based frontend.  
It consists of two crates:

- **backend**: A UI-agnostic library for process management (listing, killing, cgroup operations).
- **frontend-egui**: An egui/eframe-based desktop UI for interacting with processes.

## Features

- Live process table (60 FPS refresh)
- Sortable columns: PID, Name, CPU%, Memory, State, PPID
- Kill button for each process (SIGSTOP → SIGTERM → SIGKILL, with proper error handling)
- Non-blocking UI (process killing runs in background)
- Backend is reusable in other frontends (e.g., Tauri) without modification

## Requirements

- Linux (kernel ≥ 5.3 recommended for pidfd support)
- Rust (edition 2021)
- Cgroups v2 for cgroup killing feature

## Building

Clone the repository and build the egui frontend:

```bash
git clone https://github.com/yourusername/trash-manager.git
cd trash-manager
cargo build --release -p frontend-egui
```

## Running

Run the egui frontend:

```bash
cargo run --release -p frontend-egui
```

Or, run the built binary directly:

```bash
./target/release/frontend-egui
```

## Usage

- The main window displays a live-updating process table.
- Click column headers to sort.
- Click "Kill" to terminate a process (uses backend logic for safe termination).
- UI remains responsive during all operations.

## Architecture

- **backend**: All process logic (listing, killing, cgroup ops). No UI dependencies.
- **frontend-egui**: Calls backend functions directly for maximum speed. No IPC.

## Extending

To use the backend in another frontend (e.g., Tauri), add it as a dependency and call its API.

## License

MIT

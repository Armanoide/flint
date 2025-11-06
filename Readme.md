# macOS Service Manager

A simple Rust CLI to manage macOS services and formulas via `launchd`.  
Supports starting, stopping, checking status, and registering services at login.

---

## Features

- Start and stop user or system services
- View status of individual or all services
- Automatically register services to start at login
- Works with Homebrew formulas or custom LaunchAgents/Daemons
- Handles user-level and root-required services

---

## Installation

### Using Cargo (developer-friendly)

```bash
cargo install --path .


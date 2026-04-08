use std::{borrow::Cow, path::PathBuf};

use firecracker_rust_sdk::machine::{Machine, MachineConfig};
use uuid::Uuid;

/// Connect to a running Firecracker instance and send Ctrl+Alt+Del to
/// request a graceful shutdown.
///
/// Usage:
///   cargo run --example shutdown_vm -- /tmp/firecracker-<uuid>.sock
///
/// If no socket path is supplied defaults to /tmp/firecracker.sock.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/firecracker.sock".to_string());

    let machine_config = MachineConfig {
        vm_id: Uuid::new_v4(),
        socket_path: Cow::Owned(PathBuf::from(&socket_path)),
        exec_path: Cow::Owned(PathBuf::from("./firecracker")),
    };

    println!("Connecting to Firecracker at {socket_path}...");
    let mut vm = Machine::connect(machine_config).await?;

    let info = vm.describe_instance().await?;
    println!("Current state: {:?}", info.state);

    println!("Sending Ctrl+Alt+Del...");
    vm.send_ctrl_alt_del().await?;

    println!("Shutdown signal sent. The VM will halt momentarily.");

    Ok(())
}

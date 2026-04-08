use std::{borrow::Cow, path::PathBuf};

use firecracker_rust_sdk::machine::{Machine, MachineConfig};
use uuid::Uuid;

/// Connect to an already-running Firecracker instance and print its info.
///
/// Usage:
///   cargo run --example connect -- /tmp/fc.sock
///
/// If no socket path is given defaults to /tmp/firecracker.sock.

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
    println!("App:     {}", info.app_name);
    println!("ID:      {}", info.id);
    println!("State:   {:?}", info.state);
    println!("Version: {}", info.vmm_version);

    let version = vm.get_version().await?;
    println!("Firecracker build: {version}");

    Ok(())
}

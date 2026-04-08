use std::{borrow::Cow, path::PathBuf};

use firecracker_rust_sdk::{
    machine::{Machine, MachineConfig},
    models::{BootSource, Drive, MachineConfiguration, NetworkInterface},
};
use uuid::Uuid;

/// Minimal Firecracker microVM example.
///
/// Assumes the following files exist next to the binary:
///   ./firecracker          - Firecracker binary
///   ./vmlinux              - Uncompressed Linux kernel (e.g. vmlinux-5.10)
///   ./rootfs.ext4          - ext4 root filesystem image
///
/// Run with:
///   cargo run --example simple_vm

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vm_id = Uuid::new_v4();
    let socket_path = format!("/tmp/firecracker-{}.sock", vm_id);

    let machine_config = MachineConfig {
        vm_id,
        socket_path: Cow::Owned(PathBuf::from(&socket_path)),
        exec_path: Cow::Owned(PathBuf::from("./firecracker")),
    };

    println!("Starting Firecracker VMM (socket: {socket_path})...");
    let mut machine = Machine::start(machine_config).await?;

    println!("Configuring machine...");
    machine
        .put_machine_config(&MachineConfiguration {
            vcpu_count: 1,
            mem_size_mib: 128,
            ..Default::default()
        })
        .await?;

    println!("Configuring boot source...");
    machine
        .put_boot_source(&BootSource {
            kernel_image_path: "./vmlinux".to_string(),
            boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".to_string()),
            initrd_path: None,
        })
        .await?;

    println!("Configuring root drive...");
    machine
        .put_drive(
            "rootfs",
            &Drive {
                drive_id: "rootfs".to_string(),
                path_on_host: Some("./rootfs.ext4".to_string()),
                is_root_device: true,
                is_read_only: Some(false),
                ..Default::default()
            },
        )
        .await?;

    println!("Configuring network interface...");
    machine
        .put_network_interface(
            "eth0",
            &NetworkInterface {
                iface_id: "eth0".to_string(),
                host_dev_name: "tap0".to_string(),
                guest_mac: Some("AA:FC:00:00:00:01".to_string()),
                ..Default::default()
            },
        )
        .await?;


    println!("Starting instance...");
    let mut vm = machine.start_instance().await?;

    let info = vm.describe_instance().await?;
    println!("State: {:?}", info.state);

    println!();
    println!("VM is running!");
    println!("  Attach to the console: tmux attach -t vm_{}", info.id);
    println!("  Shut down:             cargo run --example shutdown_vm -- {socket_path}");

    Ok(())
}

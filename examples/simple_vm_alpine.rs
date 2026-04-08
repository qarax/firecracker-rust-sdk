use std::{borrow::Cow, path::PathBuf};

use firecracker_rust_sdk::{
    machine::{Machine, MachineConfig},
    models::{BootSource, Drive, MachineConfiguration},
};
use uuid::Uuid;

/// Boot an Alpine Linux microVM using Firecracker.
///
/// Before running, fetch the test images:
///   ./examples/fetch_test_images.sh
///
/// This will download:
///   - vmlinux-alpine   (Alpine-compatible uncompressed kernel)
///   - alpine.ext4      (minimal Alpine root filesystem, ~40 MB)
///
/// Run with:
///   cargo run --example simple_vm_alpine

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_dir = PathBuf::from("examples/images");
    let kernel_path = image_dir.join("vmlinux-alpine");
    let rootfs_path = image_dir.join("alpine.ext4");

    if !kernel_path.exists() || !rootfs_path.exists() {
        eprintln!("Error: Required images not found!");
        eprintln!();
        eprintln!("Please run:");
        eprintln!("  ./examples/fetch_test_images.sh");
        eprintln!();
        eprintln!("Or place these files manually:");
        eprintln!("  examples/images/vmlinux-alpine");
        eprintln!("  examples/images/alpine.ext4");
        return Err("Missing required VM images".into());
    }

    let vm_id = Uuid::new_v4();
    let socket_path = format!("/tmp/firecracker-alpine-{}.sock", vm_id);

    let machine_config = MachineConfig {
        vm_id,
        socket_path: Cow::Owned(PathBuf::from(&socket_path)),
        exec_path: Cow::Owned(PathBuf::from("./firecracker")),
    };

    println!("Starting Firecracker (socket: {socket_path})...");
    let mut machine = Machine::start(machine_config).await?;

    println!("Configuring machine...");
    machine
        .put_machine_config(&MachineConfiguration {
            vcpu_count: 2,
            mem_size_mib: 256,
            ..Default::default()
        })
        .await?;

    println!("Configuring boot source...");
    machine
        .put_boot_source(&BootSource {
            kernel_image_path: kernel_path.to_string_lossy().to_string(),
            boot_args: Some("console=ttyS0 root=/dev/vda ro reboot=k panic=1 pci=off".to_string()),
            initrd_path: None,
        })
        .await?;

    println!("Configuring root drive...");
    machine
        .put_drive(
            "rootfs",
            &Drive {
                drive_id: "rootfs".to_string(),
                path_on_host: Some(rootfs_path.to_string_lossy().to_string()),
                is_root_device: true,
                is_read_only: Some(true),
                ..Default::default()
            },
        )
        .await?;

    println!("Starting instance...");
    let mut vm = machine.start_instance().await?;

    let info = vm.describe_instance().await?;
    println!("State: {:?}", info.state);

    println!();
    println!("Alpine VM is running!");
    println!("  Attach to the console: tmux attach -t vm_{vm_id}");
    println!("  Shut down:             cargo run --example shutdown_vm -- {socket_path}");
    println!();
    println!("Alpine login: root (no password by default)");

    Ok(())
}

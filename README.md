# Firecracker Rust SDK

A Rust SDK for interacting with the [Firecracker microVM](https://github.com/firecracker-microvm/firecracker) REST API.

This SDK is generated from the official Firecracker OpenAPI specification (`firecracker.yaml`) and extended with hand-crafted abstractions (`Machine` and `MicroVm`) to seamlessly manage Firecracker instances via Unix Domain Sockets in Rust.

## Features

- **Full API Coverage**: Generated models and endpoints covering the complete Firecracker API spec.
- **Async client**: Uses `tokio` and `hyper` to communicate efficiently with the Firecracker REST API over completely local Unix sockets.
- **High-level abstractions**: Spin up new Firecracker processes and manage their lifetimes effortlessly using the `Machine` wrapper.

## Usage

### Connecting to a running microVM
```rust
use std::{borrow::Cow, path::PathBuf};
use firecracker_rust_sdk::machine::{Machine, MachineConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MachineConfig {
        vm_id: Uuid::new_v4(),
        socket_path: Cow::Owned(PathBuf::from("/tmp/firecracker.sock")),
        exec_path: Cow::Owned(PathBuf::from("/usr/local/bin/firecracker")),
    };

    let mut vm = Machine::connect(config).await?;
    let info = vm.describe_instance().await?;
    println!("VM State: {:?}", info.state);
    
    Ok(())
}
```

## Examples

We provide several standalone examples demonstrating how to boot and control microVMs in the `examples/` directory.

### Quick Start with Alpine Linux

1. Download the Firecracker binary:
```bash
wget https://github.com/firecracker-microvm/firecracker/releases/download/v1.10.1/firecracker-v1.10.1-x86_64 -O firecracker
chmod +x firecracker
```

2. Fetch the test images (Alpine kernel & rootfs) via our helper script:
```bash
./examples/fetch_test_images.sh
```

3. Run the complete Alpine booting example:
```bash
cargo run --example simple_vm_alpine
```

4. You can attach to the microVM console through the tmux session it spawned (the example output will provide the exact command, e.g., `tmux attach -t vm_<uuid>`).

### Available Examples

- `cargo run --example simple_vm_alpine` – Downloads an Alpine kernel/rootfs and fully boots an instance.
- `cargo run --example simple_vm` – A minimal example template requiring you to provide your own kernel/rootfs (`vmlinux` and `rootfs.ext4`) and set up a TAP networking device.
- `cargo run --example connect -- /tmp/firecracker.sock` – Connects to a running instance and prints diagnostics.
- `cargo run --example shutdown_vm -- /tmp/firecracker.sock` – Connects to a running instance and safely shuts it down by sending Ctrl+Alt+Del.

See [`examples/README.md`](examples/README.md) for more details.

## Development

The SDK models and base API clients are generated from `firecracker.yaml`.
If you need to update the API version:

```bash
# Downloads the openapi-generator-cli and re-generates the code
make regenerate
```

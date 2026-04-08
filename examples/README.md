# Firecracker Rust SDK – Examples

These examples show how to use the SDK to launch and manage Firecracker microVMs.

## Prerequisites

- [`firecracker`](https://github.com/firecracker-microvm/firecracker/releases) binary in `./firecracker` (or on `$PATH`)
- `tmux` installed (the SDK launches Firecracker in a tmux session)
- A TAP device `tap0` for the `simple_vm` example:
  ```bash
  sudo ip tuntap add tap0 mode tap
  sudo ip addr add 172.16.0.1/24 dev tap0
  sudo ip link set tap0 up
  ```

## Fetch test images

```bash
./examples/fetch_test_images.sh
```

This downloads a Firecracker-CI kernel (`vmlinux-alpine`) and root filesystem (`alpine.ext4`) into `examples/images/`.

## Examples

| Example | Description |
|---------|-------------|
| `connect` | Connect to a running Firecracker socket and print instance info |
| `simple_vm` | Boot a minimal microVM (BYO kernel + rootfs) |
| `simple_vm_alpine` | Boot an Alpine Linux microVM using the fetched images |
| `shutdown_vm` | Send Ctrl+Alt+Del to gracefully halt a running microVM |

### `connect`

```bash
cargo run --example connect -- /tmp/firecracker-<uuid>.sock
```

### `simple_vm`

Point `KERNEL` and `ROOTFS` at your own images:

```bash
cargo run --example simple_vm
```

Edit `examples/simple_vm.rs` to change the kernel/rootfs paths.

### `simple_vm_alpine`

```bash
./examples/fetch_test_images.sh
cargo run --example simple_vm_alpine
```

Attach to the console:

```bash
tmux attach -t vm_<uuid>
```

### `shutdown_vm`

```bash
cargo run --example shutdown_vm -- /tmp/firecracker-<uuid>.sock
```

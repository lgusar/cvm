# cvm

A CLI tool written in Rust for provisioning VMs using cloud images.

## Setup

Create a directory

```bash
sudo mkdir /var/lib/cvm
```

Set up AppArmor configuration:

```bash
echo '  /var/lib/cvm/** rwk,' | sudo tee -a /etc/apparmor.d/abstractions/libvirt-qemu
```

## Usage

```bash
cargo run <path-to-qcow2-image>
```

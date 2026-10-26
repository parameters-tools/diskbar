<div align="center">

```text
██████╗ ██╗███████╗██╗  ██╗██████╗  █████╗ ██████╗ 
██╔══██╗██║██╔════╝██║ ██╔╝██╔══██╗██╔══██╗██╔══██╗
██║  ██║██║███████╗█████╔╝ ██████╔╝███████║██████╔╝
██║  ██║██║╚════██║██╔═██╗ ██╔══██╗██╔══██║██╔══██╗
██████╔╝██║███████║██║  ██╗██████╔╝██║  ██║██║  ██║
╚═════╝ ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
```

# 💾 DISKBAR

**A blazing-fast disk usage visualizer with clean progress bars, powered by direct Linux kernel syscalls.**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Linux](https://img.shields.io/badge/linux-%23FCC624.svg?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Speed](https://img.shields.io/badge/speed-~1ms-brightgreen.svg?style=for-the-badge)](#-benchmarks)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=for-the-badge)](#-license)

[Features](#-features) •
[Install](#-install) •
[Usage](#-usage) •
[Benchmarks](#-benchmarks) •
[How It Works](#-how-it-works) •
[Limitations](#-limitations) •
[License](#-license)

</div>

---

```text
💾 DISK USAGE SUMMARY (Direct Kernel Syscall)
--------------------------------------------------------------------------------------
MOUNT POINT DEVICE         TYPE   SIZE   USED  AVAIL   USAGE BAR                     USE%
--------------------------------------------------------------------------------------
/           /dev/nvme0n1p2 ext4   467G   102G   341G   [████░░░░░░░░░░░░░░░░]   23%
/boot/efi   /dev/nvme0n1p1 vfat   1.0G   6.3M   1.0G   [░░░░░░░░░░░░░░░░░░░░]    1%
--------------------------------------------------------------------------------------
Tip: Run with --all (or -a) to include virtual / loop / tmpfs mounts.
```

`diskbar` is a modern, drop-in visual alternative to `df -h`. Unlike wrappers that shell out to slow external processes, `diskbar` reads `/proc/mounts` and calls the Linux kernel's `statvfs` directly — rendering a formatted, color-coded storage breakdown in around **1 millisecond**.

---

## ✨ Features

- ⚡ **Near-Instant:** ~1 ms typical runtime on an NVMe workstation — faster than GNU `df -h` itself.
- 📊 **Visual Progress Bars:** Dynamic ASCII storage bars (`█` / `░`) with automatic ANSI color thresholds (green `< 75%`, yellow `75–89%`, red `≥ 90%`).
- 🗂️ **Filesystem Types:** Shows the `type` column for every mount (ext4, btrfs, vfat, and more).
- 🧹 **Noise-Free by Default:** Only mounts backed by a real `/dev/` device are shown, which hides `tmpfs`, `efivarfs`, loop-backed snaps, overlay mounts, and other virtual clutter.
- 🎛️ **Full View with `-a`:** Inspect every mounted filesystem, including Docker overlays and RAM disks.
- 🪶 **Small Footprint:** Pure Rust plus `libc` syscalls; ~2.4 MB peak memory and no subprocesses.
- 🎨 **Automatic Alignment:** Column widths adapt to the longest mount point, device, and filesystem type.

---

## 🚀 Install

### Build from source

```bash
git clone https://github.com/parameters-tools/diskbar.git
cd diskbar
cargo build --release
sudo install -m 755 target/release/diskbar /usr/local/bin/diskbar
```

### Via Cargo

```bash
cargo install --git https://github.com/parameters-tools/diskbar
```

### Installer script

The installer requires an existing Rust toolchain:

```bash
curl -sSL https://raw.githubusercontent.com/parameters-tools/diskbar/main/install.sh | bash
```

---

## 📖 Usage

### Standard view (real physical disks only)

```bash
diskbar
```

### Detailed view (include virtual filesystems and snap loops)

```bash
diskbar -a
# or
diskbar --all
```

---

## ⚡ Benchmarks

Measured over 50 iterations on an NVMe Linux workstation:

| Command | Average runtime | Notes |
| :--- | ---: | :--- |
| **`diskbar`** | **~1 ms** | Direct `statvfs` queries |
| GNU `df -h` | 1.5–1.7 ms | Default Linux coreutil |
| Python / Node wrappers | tens of ms | Runtime initialization overhead |

Exact numbers depend on the number of mounts and storage speed. `diskbar` is consistently faster than `df -h` because it never forks or pipes.

---

## 🛠️ How It Works

Most CLI wrappers for `df` carry overhead because they:

1. Spawn a subshell (`/bin/sh` or `/bin/bash`).
2. Fork and execute `/bin/df -h`.
3. Pipe raw strings over IPC and parse lines with regex.

`diskbar` skips all of that:

1. Streams mount metadata directly from `/proc/mounts`.
2. Queries each mount with the kernel's `statvfs` system call via `libc`.
3. Formats sizes into human-readable units and renders aligned, color-coded columns.

No subshells, no parsing of another program's output, no external runtime.

---

## 🧭 Limitations

- **Linux only** — relies on `/proc/mounts` and `statvfs`.
- **Default filter is device-based** — mounts whose device does not start with `/dev/` (for example, some ZFS datasets or network filesystems) are hidden unless you pass `--all`.
- **Root permissions are not required**; the values shown match what the kernel reports to your user, same as `df`.
- **Rust 1.85+** (edition 2024).

---

## 📄 License

Dual-licensed under either the [MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.

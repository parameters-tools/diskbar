use std::cmp::max;
use std::collections::HashSet;
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem::MaybeUninit;

struct MountInfo {
    filesystem: String,
    mount_point: String,
    fs_type: String,
    size_str: String,
    used_str: String,
    avail_str: String,
    use_pct: u8,
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TIB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

    let b = bytes as f64;
    if b >= TIB {
        format!("{:.1}T", b / TIB)
    } else if b >= GIB {
        if b / GIB >= 10.0 {
            format!("{:.0}G", b / GIB)
        } else {
            format!("{:.1}G", b / GIB)
        }
    } else if b >= MIB {
        if b / MIB >= 10.0 {
            format!("{:.0}M", b / MIB)
        } else {
            format!("{:.1}M", b / MIB)
        }
    } else if b >= KIB {
        format!("{:.0}K", b / KIB)
    } else {
        format!("{}B", bytes)
    }
}

#[allow(clippy::unnecessary_cast)]
fn get_statvfs(path: &str) -> Option<(u64, u64, u64, u8)> {
    let c_path = CString::new(path).ok()?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();

    // Call Linux kernel statvfs() directly
    if unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }
    let stat = unsafe { stat.assume_init() };

    let block_size = stat.f_frsize as u64;
    let total_bytes = stat.f_blocks as u64 * block_size;
    let free_bytes = stat.f_bfree as u64 * block_size;
    let avail_bytes = stat.f_bavail as u64 * block_size;
    let used_bytes = total_bytes.saturating_sub(free_bytes);

    if total_bytes == 0 {
        return None;
    }

    // Standard formula used by GNU coreutils 'df'
    let non_root_total = used_bytes + avail_bytes;
    let use_pct = if non_root_total > 0 {
        ((used_bytes as f64 / non_root_total as f64) * 100.0).round() as u8
    } else {
        0
    };

    Some((total_bytes, used_bytes, avail_bytes, use_pct))
}

fn make_bar(pct: u8, width: usize) -> String {
    let filled = (pct as usize * width) / 100;
    let unfilled = width.saturating_sub(filled);

    let color = if pct >= 90 {
        "\x1b[31m" // Red
    } else if pct >= 75 {
        "\x1b[33m" // Yellow
    } else {
        "\x1b[32m" // Green
    };
    let reset = "\x1b[0m";

    let bar = format!(
        "{}{}{}",
        "█".repeat(filled),
        "\x1b[90m",
        "░".repeat(unfilled)
    );
    format!("{}[{}{}]", color, bar, reset)
}

fn main() {
    let show_all = env::args().any(|arg| arg == "--all" || arg == "-a");

    // 1. Read /proc/mounts directly from the Linux kernel (Zero external processes!)
    let file = match File::open("/proc/mounts") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ Failed to open /proc/mounts: {}", e);
            std::process::exit(1);
        }
    };

    let reader = BufReader::new(file);
    let mut mounts: Vec<MountInfo> = Vec::new();
    let mut seen_mount_points: HashSet<String> = HashSet::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let filesystem = parts[0].to_string();
        let mount_point = parts[1].to_string();
        let fs_type = parts[2].to_string();

        // Avoid duplicate mount points
        if seen_mount_points.contains(&mount_point) {
            continue;
        }

        // Filtering:
        // By default, only show real disks: starts with /dev/ and not /dev/loop (snaps)
        let is_real_disk = filesystem.starts_with("/dev/") && !filesystem.starts_with("/dev/loop");
        if !show_all && !is_real_disk {
            continue;
        }

        // Query statvfs directly from the kernel
        if let Some((total, used, avail, use_pct)) = get_statvfs(&mount_point) {
            seen_mount_points.insert(mount_point.clone());
            mounts.push(MountInfo {
                filesystem,
                mount_point,
                fs_type,
                size_str: format_bytes(total),
                used_str: format_bytes(used),
                avail_str: format_bytes(avail),
                use_pct,
            });
        }
    }

    if mounts.is_empty() {
        println!("No matching disks found. Run with --all to see all mounts.");
        return;
    }

    // Dynamic width calculation
    let mount_w = mounts
        .iter()
        .fold(11, |acc, m| max(acc, m.mount_point.len()));
    let dev_w = mounts.iter().fold(6, |acc, m| max(acc, m.filesystem.len()));
    let type_w = mounts.iter().fold(4, |acc, m| max(acc, m.fs_type.len()));
    let total_w = mount_w + dev_w + type_w + 57;
    let divider = "-".repeat(total_w);

    // 2. Render summary
    println!();
    println!("💾 \x1b[1mDISK USAGE SUMMARY (Direct Kernel Syscall)\x1b[0m");
    println!("{}", divider);
    println!(
        "\x1b[1m{:<mw$} {:<dw$} {:<tw$} {:>6} {:>6} {:>6}   {:<28} {:>5}\x1b[0m",
        "MOUNT POINT",
        "DEVICE",
        "TYPE",
        "SIZE",
        "USED",
        "AVAIL",
        "USAGE BAR",
        "USE%",
        mw = mount_w,
        dw = dev_w,
        tw = type_w
    );
    println!("{}", divider);

    for m in &mounts {
        let bar = make_bar(m.use_pct, 20);
        println!(
            "{:<mw$} {:<dw$} {:<tw$} {:>6} {:>6} {:>6}   {} {:>4}%",
            m.mount_point,
            m.filesystem,
            m.fs_type,
            m.size_str,
            m.used_str,
            m.avail_str,
            bar,
            m.use_pct,
            mw = mount_w,
            dw = dev_w,
            tw = type_w
        );
    }

    println!("{}", divider);
    if !show_all {
        println!(
            "\x1b[90mTip: Run with --all (or -a) to include virtual / loop / tmpfs mounts.\x1b[0m\n"
        );
    } else {
        println!();
    }
}

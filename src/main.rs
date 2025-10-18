use std::ffi::{CString, OsString};
use std::io;
use std::path::{Path, PathBuf};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use clap::Parser;

const DEFAULT_BUF_MB: usize = 5;

#[derive(Parser, Debug)]
#[command(name = "fc", version, about = "Fast counter using getdents64")]
struct Args {
    /// Path to traverse (default .)
    #[arg(value_name = "PATH", default_value = ".")]
    path: PathBuf,

    /// Recursively walk subdirectories
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,

    /// Count directories (instead of files)
    #[arg(short = 'D', long = "dirs")]
    count_dirs: bool,

    /// Count both files and directories
    #[arg(short = 'A', long = "all")]
    count_all: bool,

    /// Buffer size in MiB for SYS_getdents64 (default 5)
    #[arg(short = 'b', long = "buf-mb")]
    buf_mb: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let buf_mb = args.buf_mb.unwrap_or(DEFAULT_BUF_MB);
    let mut buf = vec![0u8; buf_mb.saturating_mul(1024 * 1024)];

    match count_entries(&args.path, args.recursive, args.count_dirs, args.count_all, &mut buf) {
        Ok(n) => {
            println!("{}", n);
        }
        Err(e) => {
            eprintln!("fc: {}", e);
            std::process::exit(1);
        }
    }
}

struct FdGuard(i32);
impl Drop for FdGuard {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe {
                libc::close(self.0);
            }
        }
    }
}

fn count_entries(dir: &Path, recursive: bool, count_dirs: bool, count_all: bool, buf: &mut Vec<u8>) -> io::Result<u128> {
    let c_path = CString::new(dir.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains NUL"))?;
    let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let _guard = FdGuard(fd);

    let mut total: u128 = 0;

    loop {
        let nread = unsafe {
            libc::syscall(
                libc::SYS_getdents64,
                fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
            )
        } as isize;

        if nread == -1 {
            return Err(io::Error::last_os_error());
        }
        if nread == 0 {
            break;
        }

        let mut bpos = 0usize;
        while bpos < nread as usize {
            // linux_dirent64 layout:
            // u64 d_ino (0) | i64 d_off (8) | u16 d_reclen (16) | u8 d_type (18) | char d_name[] (19..)
            if bpos + 19 > nread as usize {
                break; // stop at a corrupted record
            }
            let reclen = u16::from_ne_bytes([buf[bpos + 16], buf[bpos + 17]]) as usize;
            if reclen == 0 || bpos + reclen > nread as usize {
                break;
            }

            let d_type = buf[bpos + 18];
            let name_start = bpos + 19;
            let name_end = bpos + reclen;

            // d_name is a NUL-terminated C string
            let name_slice = &buf[name_start..name_end];
            let nul_pos = name_slice
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_slice.len());
            let name = &name_slice[..nul_pos];

            // Skip entries with zero inode (like the C++ version) and "." / ".."
            let d_ino = u64::from_ne_bytes([
                buf[bpos + 0],
                buf[bpos + 1],
                buf[bpos + 2],
                buf[bpos + 3],
                buf[bpos + 4],
                buf[bpos + 5],
                buf[bpos + 6],
                buf[bpos + 7],
            ]);
            if d_ino == 0 {
                bpos += reclen;
                continue;
            }
            if name == b"." || name == b".." {
                bpos += reclen;
                continue;
            }

            let is_dir = d_type == libc::DT_DIR;
            let is_file = d_type == libc::DT_REG;

            // Count files/directories according to flags
            if count_all {
                if is_file || is_dir {
                    total += 1;
                }
            } else if count_dirs {
                if is_dir {
                    total += 1;
                }
            } else {
                if is_file {
                    total += 1;
                }
            }

            // Recurse: only into real directories and only when the flag is set
            if recursive && is_dir {
                let mut child = dir.to_path_buf();
                let name_os = OsString::from_vec(name.to_vec());
                child.push(name_os);
                total += count_entries(&child, recursive, count_dirs, count_all, buf)?;
            }

            bpos += reclen;
        }
    }

    Ok(total)
}

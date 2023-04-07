#![feature(byte_slice_trim_ascii)]

use crate::utils::{env_var, env_var_or_empty};
use bstr::ByteSlice;
use format_bytes::{format_bytes, write_bytes};
use prompt::display_prompt;
use std::ffi::OsStr;
use std::io::{stderr, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

mod path_fmt;
mod prompt;
mod utils;

fn main() {
    let args = std::env::args_os();
    if args.len() <= 1 {
        display_prompt();
    } else {
        let args: Vec<Vec<u8>> = args.map(|s| s.into_vec()).collect();
        if args.len() != 2 {
            show_help();
            return;
        }
        match args[1].as_slice() {
            b"-v" | b"--version" => println!(concat!("focused_prompt ", env!("CARGO_PKG_VERSION"))),
            b"install" => install_prompt(),
            _ => show_help(),
        }
    }
}

#[cfg(target_os = "linux")]
fn install_prompt() {
    let stderr = &mut stderr();
    let shell = match env_var(b"SHELL") {
        None => {
            let _ = write_bytes!(stderr, b"SHELL environment variable not set");
            return;
        }
        Some(s) => s,
    };

    let shell_name = shell
        .rsplit_once_str(b"/")
        .unwrap_or((&shell[..], &shell[..]))
        .1;
    match shell_name {
        b"fish" => {
            let is_root = unsafe { libc::getuid() } == 0;

            let main_user;
            let bin_dir;
            let bin_path;
            let fish_func_dir;
            let fish_func_path;

            // Install globally if root
            if is_root {
                let _ = write_bytes!(stderr, b"User is root, installing globally");

                let sudo_user = env_var_or_empty(b"SUDO_USER");
                if sudo_user.is_empty() {
                    let _ = write_bytes!(
                        stderr,
                        b"SUDO_USER environment variable not set, not setting a main user"
                    );
                    return;
                } else {
                    main_user = Some(sudo_user);
                }

                bin_dir = b"/usr/local/bin".to_vec();
                bin_path = b"/usr/local/bin/focused_prompt".to_vec();
                fish_func_dir = b"/etc/fish/functions".to_vec();
                fish_func_path = b"/etc/fish/functions/fish_prompt.fish".to_vec();
            } else {
                let _ = write_bytes!(
                    stderr,
                    b"User is not root, installing for current user only"
                );

                let home = env_var(b"HOME").expect("HOME environment variable not set");
                main_user =
                    env_var(b"USER").and_then(|u| if u.is_empty() { None } else { Some(u) });
                bin_dir = format_bytes!(b"{}/.local/bin", &home);
                bin_path = format_bytes!(b"{}/focused_prompt", bin_dir);
                fish_func_dir = format_bytes!(b"{}/.config/fish/functions", &home);
                fish_func_path = format_bytes!(b"{}/fish_prompt.fish", fish_func_dir);
            }

            // Copy the binary to /usr/local/bin / ~/.local/bin
            if std::path::PathBuf::from(OsStr::from_bytes(b"/proc/self/exe"))
                .read_link()
                .unwrap()
                .as_os_str()
                .as_bytes()
                == bin_path
            {
                let _ = write_bytes!(stderr, b"Already running from {}, skipping...\n", bin_path);
            } else {
                std::fs::create_dir_all(OsStr::from_bytes(&bin_dir)).unwrap();
                std::fs::copy(
                    OsStr::from_bytes(b"/proc/self/exe"),
                    OsStr::from_bytes(&bin_path),
                )
                .unwrap();
            }

            // Create the fish_prompt function
            std::fs::create_dir_all(OsStr::from_bytes(&fish_func_dir)).unwrap();
            let function_file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(OsStr::from_bytes(&fish_func_path))
                .unwrap();
            let mut function_file = std::io::BufWriter::new(function_file);
            function_file
                .write_all(b"function fish_prompt\n\x20\x20\x20\x20FP_STATUS=$status ")
                .unwrap();
            if let Some(main_user) = main_user {
                write_bytes!(&mut function_file, b"FP_MAIN_USER={} ", &main_user).unwrap();
            }
            write_bytes!(&mut function_file, b"FP_COLS=$COLUMNS {}\nend\n", &bin_path).unwrap();
        }
        _ => {
            let _ = write_bytes!(
                stderr,
                b"Automatic install for shell '{}' not supported, \
                see https://github.com/Wazzaps/focused_prompt#installation for more info\n",
                shell_name,
            );
        }
    }
}

#[cfg(target_os = "macos")]
fn install_prompt() {
    let _ = write_bytes!(
        stderr,
        b"Automatic install not supported on macOS, \
          see https://github.com/Wazzaps/focused_prompt#installation for more info"
    );
}

#[cfg(target_os = "windows")]
fn install_prompt() {
    let _ = write_bytes!(
        stderr,
        b"Automatic install not supported on Windows, \
          see https://github.com/Wazzaps/focused_prompt#installation for more info"
    );
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn install_prompt() {
    let _ = write_bytes!(
        stderr,
        b"Automatic install not supported on this platform, \
          see https://github.com/Wazzaps/focused_prompt#installation for more info"
    );
}

fn show_help() {
    const HELP_STR: &[u8] = b"Usage: focused_prompt [OPTION]\n\
Display a fancy yet minimal prompt.\n\
\n\
\x20\x20\x20\x20install         install the prompt into the default shell\n\
\x20\x20\x20\x20-h, --help      display this help and exit\n\
\x20\x20\x20\x20-v, --version   output version information and exit\n\
\n";
    let mut stderr = stderr();
    let _ = stderr.write_all(HELP_STR);
}

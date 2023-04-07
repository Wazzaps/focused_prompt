use crate::path_fmt;
use crate::utils::{env_var, env_var_or_empty};
use format_bytes::write_bytes;
use std::io::Write;
use std::process::Command;

fn add_hostname(into: &mut Vec<u8>, is_connected_via_ssh: bool) {
    if is_connected_via_ssh {
        let hostname = Command::new("hostname").output().unwrap().stdout;
        into.push(b'@');
        into.extend_from_slice(hostname.trim_ascii());
    }
}

pub fn display_prompt() {
    let status_code = env_var(b"FP_STATUS")
        .map(|v| atoi::atoi(&v).expect("FP_STATUS is not a number"))
        .unwrap_or(0); // Assume success if status code is not provided
    let cols = env_var(b"FP_COLS")
        .map(|v| atoi::atoi(&v).expect("FP_COLS is not a number"))
        .unwrap_or(80); // Assume 80 cols if not specified

    // TODO: Display jobs
    // let jobs = env_var_or_empty(b"FP_JOBS");

    let main_user = env_var_or_empty(b"FP_MAIN_USER");
    let user = env_var(b"USER").unwrap_or(b"unk_user".to_vec());
    let is_connected_via_ssh = env_var_or_empty(b"SSH_CLIENT") != b"";

    const BOLD_TEXT: &[u8] = b"\x1b[1m";
    const ARROW: &[u8] = b"\xee\x82\xb0";

    const RED_BG_DARK_TEXT_THEN_SPACE: &[u8] = b"\x1b[48;2;170;17;17m\x1b[38;2;17;17;17m ";
    const SPACE_THEN_RED_TO_DARK: &[u8] = b" \x1b[48;2;17;17;17m\x1b[38;2;170;17;17m";

    const YELLOW_BG_DARK_TEXT_THEN_SPACE: &[u8] = b"\x1b[48;2;170;170;17m\x1b[38;2;17;17;17m ";
    const SPACE_THEN_YELLOW_TO_DARK: &[u8] = b" \x1b[48;2;17;17;17m\x1b[38;2;170;170;17m";

    const DARK_BG_WHITE_TEXT_THEN_SPACE: &[u8] = b"\x1b[37m\x1b[48;2;17;17;17m ";
    const DARK_TEXT_COLOR: &[u8] = b"\x1b[38;2;17;17;17m";

    const SUCCESS_TEXT_COLOR: &[u8] = b"\x1b[38;2;17;170;17m";
    const FAILURE_TEXT_COLOR: &[u8] = b"\x1b[38;2;170;68;17m";
    const SUCCESS_BG_COLOR: &[u8] = b"\x1b[48;2;17;170;17m";
    const FAILURE_BG_COLOR: &[u8] = b"\x1b[48;2;170;68;17m";
    const RESET_ALL_COLORS: &[u8] = b"\x1b[m";

    let mut prompt = vec![];
    // All of the prompt is bold
    prompt.extend_from_slice(BOLD_TEXT);

    // Username and hostname
    if user == b"root" {
        // Red arrow for root
        prompt.extend_from_slice(RED_BG_DARK_TEXT_THEN_SPACE);
        prompt.extend_from_slice(&user);
        add_hostname(&mut prompt, is_connected_via_ssh);
        prompt.extend_from_slice(SPACE_THEN_RED_TO_DARK);
        if !is_connected_via_ssh {
            prompt.extend_from_slice(ARROW);
        }
    } else if user != main_user || is_connected_via_ssh {
        // Yellow arrow for non-main users
        prompt.extend_from_slice(YELLOW_BG_DARK_TEXT_THEN_SPACE);
        prompt.extend_from_slice(&user);
        add_hostname(&mut prompt, is_connected_via_ssh);
        prompt.extend_from_slice(SPACE_THEN_YELLOW_TO_DARK);
        if !is_connected_via_ssh {
            prompt.extend_from_slice(ARROW);
        }
    }

    // Current working directory
    // Bold, #fff foreground, #111 background
    prompt.extend_from_slice(DARK_BG_WHITE_TEXT_THEN_SPACE);
    path_fmt::format_path(&mut prompt, cols);

    if is_connected_via_ssh {
        // -- Simple prompt, doesn't use powerline font --
        prompt.push(b' ');
        prompt.extend_from_slice(RESET_ALL_COLORS);
        prompt.push(b' ');
        if status_code == 0 {
            prompt.extend_from_slice(SUCCESS_TEXT_COLOR);
            prompt.push(b'%');
        } else {
            prompt.extend_from_slice(FAILURE_TEXT_COLOR);
            write_bytes!(&mut prompt, b"{}", status_code).unwrap();
        }
    } else {
        // -- Fancy prompt, uses powerline font --
        prompt.extend_from_slice(DARK_TEXT_COLOR);
        if status_code == 0 {
            prompt.extend_from_slice(SUCCESS_BG_COLOR);
            prompt.extend_from_slice(ARROW);
            prompt.extend_from_slice(RESET_ALL_COLORS);
            prompt.extend_from_slice(BOLD_TEXT);
            prompt.extend_from_slice(SUCCESS_TEXT_COLOR);
        } else {
            prompt.extend_from_slice(FAILURE_BG_COLOR);
            prompt.extend_from_slice(ARROW);
            write_bytes!(&mut prompt, b"{}", status_code).unwrap();
            prompt.extend_from_slice(RESET_ALL_COLORS);
            prompt.extend_from_slice(BOLD_TEXT);
            prompt.extend_from_slice(FAILURE_TEXT_COLOR);
        }
        prompt.extend_from_slice(ARROW);
    }
    prompt.extend_from_slice(RESET_ALL_COLORS);
    prompt.push(b' ');

    let _ = std::io::stdout().write_all(prompt.as_slice());
}

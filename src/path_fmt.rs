use crate::utils::env_var;
use bstr::ByteVec;
use format_bytes::format_bytes;
use std::cmp::max;
use std::env::current_dir;
use std::os::unix::ffi::OsStrExt;

fn colorize_path(into: &mut Vec<u8>, input_path: Vec<u8>) {
    const COLORS: &[&[u8]] = &[
        b"247;160;212",
        b"247;160;160",
        b"247;212;160",
        b"230;247;160",
        b"177;247;160",
        b"160;247;195",
        b"160;247;247",
        b"160;195;247",
        b"177;160;247",
        b"230;160;247",
    ];
    let mut result = vec![];
    for (i, word) in input_path.split(|c| *c == b'/').enumerate() {
        result.extend(format_bytes!(
            b"\x1b[38;2;{}m{}/",
            COLORS[i % COLORS.len()],
            word
        ));
    }
    result.pop();
    into.extend_from_slice(&result);
}

fn replace_home_in_path(path: &mut Vec<u8>) {
    let home_dir = env_var(b"HOME");
    if let Some(home_dir) = home_dir {
        if path.starts_with(&home_dir) {
            path.replace_range(0..home_dir.len(), b"~");
            return;
        }
    }

    if path.starts_with(b"/home/") {
        path.replace_range(0..6, b"~");
        return;
    }

    if path.starts_with(b"/root") {
        path.replace_range(0..5, b"~root");
    }
}

fn remove_vowels(mut input_path: Vec<u8>, target_len: usize) -> Vec<u8> {
    let mut left = (input_path.len() as i64 - target_len as i64).max(0);
    const VOWELS: &[u8] = &[b'a', b'e', b'i', b'o', b'u'];
    input_path.retain(|c| {
        if left > 0 && VOWELS.contains(c) {
            left -= 1;
            false
        } else {
            true
        }
    });
    input_path
}

fn shorten_words(input_path: Vec<u8>, target_len: usize) -> Vec<u8> {
    let path = input_path.split(|c| *c == b'/').collect::<Vec<_>>();
    let curr_len = input_path.len();
    let mut left = max(curr_len as i64 - target_len as i64, 0);

    struct WordState {
        word: Vec<u8>,
        left_to_remove: i64,
        removed: i64,
        importance_factor: i32,
    }

    let mut path_cand = path
        .iter()
        .enumerate()
        .map(|(i, word)| WordState {
            word: word.to_vec(),
            left_to_remove: max(word.len() as i64 - 1, 0),
            importance_factor: if i != path.len() - 1 { 1 } else { 5 },
            removed: 0,
        })
        .collect::<Vec<_>>();

    while left > 0 {
        let ind = path
            .iter()
            .enumerate()
            .max_by_key(|(i, _)| {
                path_cand[*i].left_to_remove / path_cand[*i].importance_factor as i64
            })
            .unwrap()
            .0;
        if path_cand[ind].left_to_remove == 0 {
            break;
        }
        path_cand[ind].left_to_remove -= 1;
        path_cand[ind].removed += 1;
        left -= 1;
    }
    path_cand
        .iter()
        .map(|word| {
            let letters_to_keep = word.word.len() - word.removed as usize;
            let start_letters = (4 * letters_to_keep) / 5;
            let end_letters = letters_to_keep - start_letters;
            format_bytes!(
                b"{}{}",
                &word.word[..start_letters],
                &word.word[word.word.len() - end_letters..]
            )
        })
        .collect::<Vec<_>>()
        .join(&b'/')
}

pub fn format_path(into: &mut Vec<u8>, cols: u32) {
    let mut path = current_dir()
        .map(|path| path.as_os_str().as_bytes().to_vec())
        .unwrap_or_else(|_| b"(unreachable)".to_vec());

    replace_home_in_path(&mut path);

    let target_len = max(30, cols as usize / 2);
    let path = if path.len() <= target_len {
        path
    } else {
        shorten_words(remove_vowels(path, target_len), target_len)
    };

    colorize_path(into, path);
}

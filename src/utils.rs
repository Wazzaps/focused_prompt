use std::ffi::OsStr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub fn env_var(name: &[u8]) -> Option<Vec<u8>> {
    std::env::var_os(OsStr::from_bytes(name)).map(|v| v.into_vec())
}

pub fn env_var_or_empty(name: &[u8]) -> Vec<u8> {
    env_var(name).unwrap_or_default()
}

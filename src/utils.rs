use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub fn env_var(name: &[u8]) -> Option<Vec<u8>> {
    std::env::var_os(OsStr::from_bytes(name)).map(|v| v.into_vec())
}

pub fn env_var_or_empty(name: &[u8]) -> Vec<u8> {
    env_var(name).unwrap_or_default()
}

pub trait SmallExpect<T> {
    fn expect2(self, msg: &[u8]) -> T;
}

impl<T> SmallExpect<T> for Option<T> {
    fn expect2(self, msg: &[u8]) -> T {
        self.unwrap_or_else(|| {
            let stderr = &mut std::io::stderr();
            let _ = stderr.write_all(b"Error: ");
            let _ = stderr.write_all(msg);
            let _ = stderr.write_all(b"\n");
            std::process::exit(1);
        })
    }
}

impl<T, E> SmallExpect<T> for Result<T, E> {
    fn expect2(self, msg: &[u8]) -> T {
        self.unwrap_or_else(|_| {
            let stderr = &mut std::io::stderr();
            let _ = stderr.write_all(b"Error: ");
            let _ = stderr.write_all(msg);
            let _ = stderr.write_all(b"\n");
            std::process::exit(1);
        })
    }
}

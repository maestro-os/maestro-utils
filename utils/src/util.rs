//! Utility functions.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::mem::size_of;
use std::ops::Shl;
use std::os::unix::ffi::OsStrExt;
use std::slice;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Reinterprets the given reference as a slice.
pub fn reinterpret<T>(val: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(val as *const _ as *const u8, size_of::<T>()) }
}

/// Returns the system's hostname.
pub fn get_hostname() -> OsString {
    let mut hostname: [u8; 4096] = [0; 4096];
    unsafe {
        libc::gethostname(hostname.as_mut_ptr() as _, hostname.len());
    }
    OsStr::from_bytes(&hostname).to_owned()
}

/// Returns the current timestamp since the Unix epoch.
pub fn get_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock panic!")
}

/// Executes the closure `f`.
///
/// If the closure returns Ok, the function returns directly.
///
/// If it returns an error, the function ensures the execution takes at least the given duration
/// `d`.
pub fn exec_wait<T, F: FnOnce() -> T>(d: Duration, f: F) -> T {
    let start = get_timestamp();
    let result = f();
    // Wait until the given amount of time is spent
    loop {
        let ts = get_timestamp();
        if ts >= start + d {
            break;
        }
        thread::sleep(ts - start);
    }
    result
}

/// Fills the given buffer with random bytes.
pub fn get_random(buf: &mut [u8]) {
    unsafe {
        libc::getrandom(buf.as_mut_ptr() as _, buf.len(), 0);
    }
}

/// A displayable number of bytes.
pub struct ByteSize(pub u64);

impl fmt::Display for ByteSize {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 1 {
            return write!(fmt, "1 byte");
        }
        // log_2(1024) = 10
        let mut order = self.0.checked_ilog2().unwrap_or(0) / 10;
        let suffix = match order {
            0 => "bytes",
            1 => "KiB",
            2 => "MiB",
            3 => "GiB",
            4 => "TiB",
            5 => "PiB",
            6 => "EiB",
            // Higher orders would overflow a `u64`
            _ => {
                order = 0;
                "bytes"
            }
        };
        let nbr = self.0 >> (10 * order as u64);
        write!(fmt, "{nbr} {suffix}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytesize() {
        assert_eq!(ByteSize(0).to_string(), "0 bytes");
        assert_eq!(ByteSize(1).to_string(), "1 byte");
        assert_eq!(ByteSize(1023).to_string(), "1023 bytes");
        assert_eq!(ByteSize(1024).to_string(), "1 KiB");
        assert_eq!(ByteSize(1025).to_string(), "1 KiB");
        assert_eq!(ByteSize(2048).to_string(), "2 KiB");
        assert_eq!(ByteSize(1024 * 1024).to_string(), "1 MiB");
        assert_eq!(ByteSize(1024 * 1024 * 1024).to_string(), "1 GiB");
        assert_eq!(ByteSize(1024 * 1024 * 1024 * 1024).to_string(), "1 TiB");
    }
}

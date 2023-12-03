//! This module implements utility functions.

use std::ffi::c_char;
use std::fmt;
use std::mem::size_of;
use std::ops::Shl;
use std::slice;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Reinterprets the given reference as a slice.
pub fn reinterpret<T>(val: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(val as *const _ as *const u8, size_of::<T>()) }
}

/// Turns the given buffer into a [`String`].
pub fn array_to_string(buf: &[c_char]) -> String {
    buf.into_iter()
        .take_while(|b| **b != 0)
        .map(|b| (*b) as u8 as char)
        .collect()
}

/// Returns the hostname of the system.
pub fn get_hostname() -> String {
    let mut hostname: [i8; 4096] = [0; 4096];
    unsafe {
        libc::gethostname(hostname.as_mut_ptr() as _, hostname.len());
        array_to_string(&hostname)
    }
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
/// If it return an error, the function ensures the execution takes at least the given duration
/// `d`.
pub fn exec_wait<T, F: FnOnce() -> T>(d: Duration, f: F) -> T {
    let start = get_timestamp();
    let result = f();
    // Waiting until the given amount of time is spent
    while get_timestamp() < start + d {
        thread::sleep(Duration::from_millis(1));
    }
    result
}

/// Fills the given buffer with random bytes.
pub fn get_random(buf: &mut [u8]) {
    unsafe {
        libc::getrandom(buf.as_mut_ptr() as _, buf.len(), 0);
    }
}

/// Computes 2^^n on unsigned integers (where `^^` is an exponent).
///
/// If n < 0, the behaviour is undefined.
pub fn pow2<T>(n: T) -> T
where
    T: From<u8> + Shl<Output = T>,
{
    T::from(1) << n
}

/// Performs the log2 operation on the given integer.
///
/// If the result is undefined, the function returns `None`.
pub fn log2(n: u64) -> Option<u64> {
    let num_bits = u64::BITS as u64;

    let n = num_bits - n.leading_zeros() as u64;
    if n > 0 {
        Some(n - 1)
    } else {
        None
    }
}

/// Structure representing a number of bytes.
pub struct ByteSize(pub u64);

impl ByteSize {
    /// Creates a size from a given number of sectors.
    pub fn from_sectors_count(cnt: u64) -> Self {
        Self(cnt * 512)
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut order = log2(self.0).unwrap_or(0) / log2(1024).unwrap();

        let suffix = match order {
            0 => "bytes",
            1 => "KiB",
            2 => "MiB",
            3 => "GiB",
            4 => "TiB",
            5 => "PiB",
            6 => "EiB",
            7 => "ZiB",
            8 => "YiB",

            _ => {
                order = 0;
                "bytes"
            }
        };

        let unit = 1024u64.pow(order as u32);
        let nbr = self.0 / unit as u64;

        write!(fmt, "{} {}", nbr, suffix)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytesize() {
        assert_eq!(ByteSize(0).to_string(), "0 bytes");
        assert_eq!(ByteSize(1).to_string(), "1 bytes");
        assert_eq!(ByteSize(1023).to_string(), "1023 bytes");
        assert_eq!(ByteSize(1024).to_string(), "1 KiB");
        assert_eq!(ByteSize(1025).to_string(), "1 KiB");
        assert_eq!(ByteSize(2048).to_string(), "2 KiB");
        assert_eq!(ByteSize(1024 * 1024).to_string(), "1 MiB");
        assert_eq!(ByteSize(1024 * 1024 * 1024).to_string(), "1 GiB");
        assert_eq!(ByteSize(1024 * 1024 * 1024 * 1024).to_string(), "1 TiB");
    }
}

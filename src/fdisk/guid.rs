//! Implementation of a Globally Unique IDentifier used in GPT partition tables.

use std::fmt;
use std::str::FromStr;

/// Type representing a Globally Unique IDentifier.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Guid(pub [u8; 16]);

impl FromStr for Guid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 36 {
            return Err(());
        }
        if s.chars().any(|c| !c.is_alphanumeric() && c != '-') {
            return Err(());
        }

        // Parse
        let iter = s
            .chars()
            .filter_map(|c| match c {
                c @ '0'..='9' => Some(c as u8 - b'0'),
                c @ 'A'..='F' => Some(c as u8 - b'A' + 10),
                c @ 'a'..='f' => Some(c as u8 - b'a' + 10),
                _ => None,
            })
            .array_chunks::<2>()
            .map(|c| c[0] * 16 + c[1])
            .enumerate();
        // Fill array while reordering
        let mut guid = Self([0; 16]);
        for (i, b) in iter {
            // Reverse necessary parts
            let index = match i {
                0..4 => 4 - i - 1,
                4..6 => 6 - i - 1 + 4,
                6..8 => 8 - i - 1 + 6,
                _ => i,
            };
            guid.0[index] = b;
        }
        Ok(guid)
    }
}

impl Guid {
    /// Generates a random GUID.
    pub fn random() -> Self {
        let mut buf = [0; 16];
        utils::util::get_random(&mut buf);
        Self(buf)
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in (0..4).rev() {
            write!(fmt, "{:02x}", self.0[i])?;
        }
        write!(fmt, "-")?;

        for i in 0..2 {
            for j in (0..2).rev() {
                write!(fmt, "{:02x}", self.0[4 + i * 2 + j])?;
            }
            write!(fmt, "-")?;
        }

        for i in 8..10 {
            write!(fmt, "{:02x}", self.0[i])?;
        }
        write!(fmt, "-")?;

        for i in 10..16 {
            write!(fmt, "{:02x}", self.0[i])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn guid_parse_valid() {
        let guid = Guid::from_str("c12a7328-f81f-11d2-ba4b-00a0c93ec93b").unwrap();
        assert_eq!(
            guid.0,
            [
                0x28, 0x73, 0x2a, 0xc1, 0x1f, 0xf8, 0xd2, 0x11, 0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e,
                0xc9, 0x3b
            ]
        );

        let guid = Guid::from_str("C12A7328-F81F-11D2-BA4B-00A0C93EC93B").unwrap();
        assert_eq!(
            guid.0,
            [
                0x28, 0x73, 0x2a, 0xc1, 0x1f, 0xf8, 0xd2, 0x11, 0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e,
                0xc9, 0x3b
            ]
        );
    }

    #[test]
    pub fn guid_parse_invalid() {
        Guid::from_str("c12a7328f81f11d2ba4b00a0c93ec93b").unwrap_err();
        Guid::from_str("c12a7328f81f11d2ba4b00a0c93ec93").unwrap_err();
        Guid::from_str("c12a7328f81f11d2ba4b00a0c93ec93$").unwrap_err();
    }
}

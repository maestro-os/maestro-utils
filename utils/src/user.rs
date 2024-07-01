//! The passwd, shadow and group files are mainly used to store respectively the users list, the
//! passwords list and the groups list.

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use libc::{gid_t, uid_t};
use rand_core::OsRng;
use std::fmt::Formatter;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::{fmt, io};

/// The path to the passwd file.
pub const PASSWD_PATH: &str = "/etc/passwd";
/// The path to the shadow file.
pub const SHADOW_PATH: &str = "/etc/shadow";
/// The path to the group file.
pub const GROUP_PATH: &str = "/etc/group";

// TODO For each files, use a backup file with the same path but with `-` appended at the end

/// Hashes the given clear password and returns it with a generated salt, in the format
/// required for the shadow file.
pub fn hash_password(pass: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(pass.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Tells whether the given password `pass` corresponds to the hashed password `hash`.
pub fn check_password(hash: &str, pass: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(pass.as_bytes(), &parsed_hash)
        .is_ok()
}

/// An error occurring when meeting an invalid entry.
#[derive(Debug)]
pub struct InvalidEntry;

/// Wrapper for [`Option`] allowing to display a value if [`Some`], or nothing if [`None`].
struct OptionDisplay<T: fmt::Display>(Option<T>);

impl<T: fmt::Display> fmt::Display for OptionDisplay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(val) => write!(f, "{val}"),
            None => Ok(()),
        }
    }
}

/// A system user, present in the `passwd` file.
pub struct User<'s> {
    /// The user's login name.
    pub login_name: &'s str,
    /// The user's encrypted password. If `x`, the password is located in the shadow file.
    pub password: &'s str,
    /// The user ID.
    pub uid: u32,
    /// The user's group ID.
    pub gid: u32,
    /// User comment.
    pub comment: &'s str,
    /// User's home path.
    pub home: &'s Path,
    /// User's command interpreter.
    pub interpreter: &'s str,
}

impl User<'_> {
    /// Deserializes entries from the given buffer `buf`.
    pub fn deserialize(buf: &str) -> impl Iterator<Item = Result<User, InvalidEntry>> {
        buf.split('\n')
            .map(|line| {
                let mut vals = line.split(':');
                let ent = User {
                    login_name: vals.next()?,
                    password: vals.next()?,
                    uid: vals.next()?.parse().ok()?,
                    gid: vals.next()?.parse().ok()?,
                    comment: vals.next()?,
                    home: Path::new(vals.next()?),
                    interpreter: vals.next()?,
                };
                if vals.next().is_some() {
                    return None;
                }
                Some(ent)
            })
            .map(|ent| ent.ok_or(InvalidEntry))
    }

    /// Check the given (not hashed) password `pass` against the current entry.
    ///
    /// If the function returns None, the callee must use the shadow entry.
    pub fn check_password(&self, pass: &str) -> Option<bool> {
        if self.password.is_empty() || self.password == "x" {
            return None;
        }
        Some(check_password(self.password, pass))
    }
}

impl fmt::Display for User<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}:{}:{}:{}:{}:{}:{}",
            self.login_name,
            self.password,
            self.uid,
            self.gid,
            self.comment,
            self.home.display(),
            self.interpreter
        )
    }
}

/// A shadow entry, present in the `shadow` file.
pub struct Shadow<'s> {
    /// The user's login name.
    pub login_name: &'s str,
    /// The user's encrypted password.
    pub password: &'s str,
    /// The date of the last password change in number of days since the Unix Epoch.
    pub last_change: u32,
    /// The minimum number of days to wait before the user becomes usable.
    pub minimum_age: Option<u32>,
    /// The maximum number of days to the password is valid. If this delay is exceeded, the user
    /// will be asked to change her password next time she logs.
    pub maximum_age: Option<u32>,
    /// The number of days before the password expires during which the user will be warned to
    /// change her password.
    pub warning_period: Option<u32>,
    /// The number of days after password expiration during which the user can still use her
    /// password. Passed this delay, she will have to contact her system administrator.
    pub inactivity_period: Option<u32>,
    /// The number of days after the Unix Epoch after which login to the user account will be
    /// denied.
    pub account_expiration: Option<u32>,
    /// Reserved field.
    pub reserved: &'s str,
}

impl Shadow<'_> {
    /// Deserializes entries from the given buffer `buf`.
    pub fn deserialize(buf: &str) -> impl Iterator<Item = Result<Shadow, InvalidEntry>> {
        buf.split('\n')
            .map(|line| {
                let mut vals = line.split(':');
                let ent = Shadow {
                    login_name: vals.next()?,
                    password: vals.next()?,
                    last_change: vals.next()?.parse().unwrap_or(0),
                    minimum_age: vals.next()?.parse().ok(),
                    maximum_age: vals.next()?.parse().ok(),
                    warning_period: vals.next()?.parse().ok(),
                    inactivity_period: vals.next()?.parse().ok(),
                    account_expiration: vals.next()?.parse().ok(),
                    reserved: vals.next()?,
                };
                if vals.next().is_some() {
                    return None;
                }
                Some(ent)
            })
            .map(|ent| ent.ok_or(InvalidEntry))
    }

    /// Check the given (not hashed) password `pass` against `self`.
    pub fn check_password(&self, pass: &str) -> bool {
        check_password(self.password, pass)
    }
}

impl fmt::Display for Shadow<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.login_name,
            self.password,
            self.last_change,
            OptionDisplay(self.minimum_age),
            OptionDisplay(self.maximum_age),
            OptionDisplay(self.warning_period),
            OptionDisplay(self.inactivity_period),
            OptionDisplay(self.account_expiration),
            self.reserved,
        )
    }
}

/// A system group, present in `group`.
pub struct Group<'s> {
    /// The group's name.
    pub group_name: &'s str,
    /// The encrypted group's password.
    pub password: &'s str,
    /// The group's ID.
    pub gid: u32,
    /// The list of user members of this group, comma-separated.
    pub users_list: &'s str,
}

impl Group<'_> {
    /// Deserializes entries from the given buffer `buf`.
    pub fn deserialize(buf: &str) -> impl Iterator<Item = Result<Group, InvalidEntry>> {
        buf.split('\n')
            .map(|line| {
                let mut vals = line.split(':');
                let ent = Group {
                    group_name: vals.next()?,
                    password: vals.next()?,
                    gid: vals.next()?.parse().ok()?,
                    users_list: vals.next()?,
                };
                if vals.next().is_some() {
                    return None;
                }
                Some(ent)
            })
            .map(|ent| ent.ok_or(InvalidEntry))
    }
}

impl fmt::Display for Group<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}:{}:{}:{}",
            self.group_name, self.password, self.gid, self.users_list
        )
    }
}

/// Writes the file at path `path` with data `data`.
pub fn write<I: IntoIterator<Item = E>, E: fmt::Display>(path: &Path, data: I) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    for line in data {
        write!(file, "{}", line)?;
    }
    Ok(())
}

/// Returns the current effective UID.
pub fn get_euid() -> uid_t {
    unsafe { libc::geteuid() }
}

/// Returns the current effective GID.
pub fn get_egid() -> gid_t {
    unsafe { libc::getegid() }
}

/// Sets the current user.
pub fn set(uid: u32, gid: u32) -> io::Result<()> {
    let result = unsafe { libc::setuid(uid) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    let result = unsafe { libc::setgid(gid) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

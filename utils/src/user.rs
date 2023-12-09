//! The passwd, shadow and group files are mainly used to store respectively the users list, the
//! passwords list and the groups list.

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

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

/// Structure representing a user. This entry is present in the passwd file.
pub struct User {
    /// The user's login name.
    pub login_name: String,
    /// The user's encrypted password. If `x`, the password is located in the shadow file.
    pub password: String,
    /// The user ID.
    pub uid: u32,
    /// The user's group ID.
    pub gid: u32,
    /// User comment.
    pub comment: String,
    /// User's home path.
    pub home: PathBuf,
    /// User's command interpreter.
    pub interpreter: String,
}

impl User {
    /// Check the given (not hashed) password `pass` against the current entry.
    ///
    /// If the function returns None, the callee must use the shadow entry.
    pub fn check_password(&self, pass: &str) -> Option<bool> {
        if self.password.is_empty() || self.password == "x" {
            return None;
        }
        Some(check_password(&self.password, pass))
    }
}

/// Structure representing a shadow entry.
pub struct Shadow {
    /// The user's login name.
    pub login_name: String,
    /// The user's encrypted password.
    pub password: String,
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
    pub reserved: String,
}

impl Shadow {
    /// Check the given (not hashed) password `pass` against the current entry.
    pub fn check_password(&self, pass: &str) -> bool {
        check_password(&self.password, pass)
    }
}

/// Structure representing a group.
pub struct Group {
    /// The group's name.
    pub group_name: String,
    /// The encrypted group's password.
    pub password: String,
    /// The group's ID.
    pub gid: u32,
    /// The list of users member of this group, comma-separated.
    pub users_list: String,
}

/// Reads and parses the file at path `path`.
fn read(path: &Path) -> io::Result<impl Iterator<Item = io::Result<Vec<String>>>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file)
        .lines()
        .map(|l| Ok(l?.split(':').map(str::to_owned).collect::<Vec<_>>())))
}

/// Writes the file at path `path` with data `data`.
fn write<const T: usize, I: IntoIterator<Item = [OsString; T]>>(
    path: &Path,
    data: I,
) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).write(true).open(path)?;
    for line in data {
        for (i, elem) in line.iter().enumerate() {
            file.write_all(elem.as_bytes())?;
            if i + 1 < line.len() {
                file.write_all(b":")?;
            }
        }
        file.write_all(b"\n")?;
    }
    Ok(())
}

/// Reads the passwd file.
///
/// `path` is the path to the file.
pub fn read_passwd(path: &Path) -> Result<Vec<User>, Box<dyn Error>> {
    read(path)?
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
            let data = data?;
            if data.len() != 7 {
                return Err(format!("Invalid entry on line `{}`", i + 1).into());
            }

            Ok(User {
                login_name: data[0].clone(),
                password: data[1].clone(),
                uid: data[2].parse::<_>()?,
                gid: data[3].parse::<_>()?,
                comment: data[4].clone(),
                home: data[5].clone().into(),
                interpreter: data[6].clone(),
            })
        })
        .collect()
}

/// Writes the passwd file.
///
/// `path` is the path to the file.
pub fn write_passwd(path: &Path, entries: &[User]) -> io::Result<()> {
    let iter = entries.iter().map(|e| {
        [
            e.login_name.clone().into(),
            e.password.clone().into(),
            e.uid.to_string().into(),
            e.gid.to_string().into(),
            e.comment.clone().into(),
            e.home.clone().into_os_string(),
            e.interpreter.clone().into(),
        ]
    });
    write(path, iter)
}

/// Reads the shadow file.
///
/// `path` is the path to the file.
pub fn read_shadow(path: &Path) -> Result<Vec<Shadow>, Box<dyn Error>> {
    read(path)?
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
            let data = data?;
            if data.len() != 9 {
                return Err(format!("Invalid entry on line `{}`", i + 1).into());
            }

            Ok(Shadow {
                login_name: data[0].clone(),
                password: data[1].clone(),
                last_change: data[2].parse::<_>().unwrap_or(0),
                minimum_age: data[3].parse::<_>().ok(),
                maximum_age: data[4].parse::<_>().ok(),
                warning_period: data[5].parse::<_>().ok(),
                inactivity_period: data[6].parse::<_>().ok(),
                account_expiration: data[7].parse::<_>().ok(),
                reserved: data[8].clone(),
            })
        })
        .collect()
}

/// Writes the shadow file.
///
/// `path` is the path to the file.
pub fn write_shadow(path: &Path, entries: &[Shadow]) -> io::Result<()> {
    let iter = entries.iter().map(|e| {
        [
            e.login_name.clone().into(),
            e.password.clone().into(),
            e.last_change.to_string().into(),
            e.minimum_age
                .as_ref()
                .map(u32::to_string)
                .unwrap_or_default()
                .into(),
            e.maximum_age
                .as_ref()
                .map(u32::to_string)
                .unwrap_or_default()
                .into(),
            e.warning_period
                .as_ref()
                .map(u32::to_string)
                .unwrap_or_default()
                .into(),
            e.inactivity_period
                .as_ref()
                .map(u32::to_string)
                .unwrap_or_default()
                .into(),
            e.account_expiration
                .as_ref()
                .map(u32::to_string)
                .unwrap_or_default()
                .into(),
            e.reserved.clone().into(),
        ]
    });
    write(path, iter)
}

/// Reads the group file.
///
/// `path` is the path to the file.
pub fn read_group(path: &Path) -> Result<Vec<Group>, Box<dyn Error>> {
    read(path)?
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
            let data = data?;
            if data.len() != 4 {
                return Err(format!("Invalid entry on line `{}`", i + 1).into());
            }

            Ok(Group {
                group_name: data[0].clone(),
                password: data[1].clone(),
                gid: data[2].parse::<_>()?,
                users_list: data[3].clone(),
            })
        })
        .collect()
}

/// Writes the group file.
///
/// `path` is the path to the file.
pub fn write_group(path: &Path, entries: &[Group]) -> io::Result<()> {
    let iter = entries.iter().map(|e| {
        [
            e.group_name.clone().into(),
            e.password.clone().into(),
            e.gid.to_string().into(),
            e.users_list.clone().into(),
        ]
    });
    write(path, iter)
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

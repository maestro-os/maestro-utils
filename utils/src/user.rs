//! The passwd, shadow and group files are mainly used to store respectively the users list, the
//! passwords list and the groups list.

use std::error::Error;
use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;
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

extern "C" {
    fn setuid(uid: u32) -> i32;
    fn setgid(uid: u32) -> i32;

    //fn hash_pass(pass: *const i8) -> *const i8;
    fn check_pass(pass: *const i8, hashed: *const i8) -> i32;

    fn free(ptr: *mut c_void);
}

/// Hashes the given clear password and returns it with a generated salt, in the format
/// required for the shadow file.
pub fn hash_password(pass: &str) -> String {
    /*let pass = CString::new(pass).unwrap();

    let s = unsafe {
        let ptr = hash_pass(pass.as_ptr());

        let s = CStr::from_ptr(ptr).to_string_lossy().to_string();

        free(ptr as _);

        s
    };

    s*/
    "TODO".into()
}

// TODO For each files, use a backup file with the same path but with `-` appended at the end

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

        let pass = CString::new(pass).unwrap();
        let password = CString::new(self.password.clone()).unwrap();
        let result = unsafe { check_pass(pass.as_ptr(), password.as_ptr()) != 0 };

        Some(result)
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
        let pass = CString::new(pass).unwrap();
        let password = CString::new(self.password.clone()).unwrap();

        unsafe { check_pass(pass.as_ptr(), password.as_ptr()) != 0 }
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
fn read(path: &Path) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut data = vec![];

    for l in BufReader::new(file).lines() {
        data.push(l?.split(':').map(|s| s.to_owned()).collect::<_>());
    }

    Ok(data)
}

/// Writes the file at path `path` with data `data`.
fn write(path: &Path, data: &[Vec<OsString>]) -> io::Result<()> {
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
    let entries: Vec<Vec<OsString>> = entries
        .iter()
        .map(|e| {
            vec![
                e.login_name.clone().into(),
                e.password.clone().into(),
                format!("{}", e.uid).into(),
                format!("{}", e.gid).into(),
                e.comment.clone().into(),
                e.home.clone().into(),
                e.interpreter.clone().into(),
            ]
        })
        .collect();

    write(path, &entries)
}

/// Reads the shadow file.
///
/// `path` is the path to the file.
pub fn read_shadow(path: &Path) -> Result<Vec<Shadow>, Box<dyn Error>> {
    read(path)?
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
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
    let entries: Vec<Vec<OsString>> = entries
        .iter()
        .map(|e| {
            vec![
                e.login_name.clone().into(),
                e.password.clone().into(),
                format!("{}", e.last_change).into(),
                e.minimum_age
                    .map(|v| format!("{}", v))
                    .unwrap_or("".to_owned())
                    .into(),
                e.maximum_age
                    .map(|v| format!("{}", v))
                    .unwrap_or("".to_owned())
                    .into(),
                e.warning_period
                    .map(|v| format!("{}", v))
                    .unwrap_or("".to_owned())
                    .into(),
                e.inactivity_period
                    .map(|v| format!("{}", v))
                    .unwrap_or("".to_owned())
                    .into(),
                e.account_expiration
                    .map(|v| format!("{}", v))
                    .unwrap_or("".to_owned())
                    .into(),
                e.reserved.clone().into(),
            ]
        })
        .collect();

    write(path, &entries)
}

/// Reads the group file.
///
/// `path` is the path to the file.
pub fn read_group(path: &Path) -> Result<Vec<Group>, Box<dyn Error>> {
    read(path)?
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
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
    let entries: Vec<Vec<OsString>> = entries
        .iter()
        .map(|e| {
            vec![
                e.group_name.clone().into(),
                e.password.clone().into(),
                format!("{}", e.gid).into(),
                e.users_list.clone().into(),
            ]
        })
        .collect();

    write(path, &entries)
}

/// Sets the current user.
pub fn set(uid: u32, gid: u32) -> Result<(), Box<dyn Error>> {
    let result = unsafe { setuid(uid) };
    if result < 0 {
        return Err("Failed to set UID!".into());
    }

    let result = unsafe { setgid(gid) };
    if result < 0 {
        return Err("Failed to set GID!".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_check_pass0() {
        let pass = CString::new("123").unwrap();
        let password = CString::new("$6$sn0mUlqBuPqbywGS$aq0m2R66gj/Q6DdPfRkOzGDs15CY4Tq40Bju64b8kwbk2RWvXgKDhDiNK4qcJk8bUFY6zBcfJ2usxhd3lA7RC1").unwrap();
        let result = unsafe { check_pass(pass.as_ptr(), password.as_ptr()) != 0 };

        assert!(result);
    }

    #[test]
    fn test_check_pass1() {
        let pass = CString::new("123456").unwrap();
        let password = CString::new("$6$sn0mUlqBuPqbywGS$aq0m2R66gj/Q6DdPfRkOzGDs15CY4Tq40Bju64b8kwbk2RWvXgKDhDiNK4qcJk8bUFY6zBcfJ2usxhd3lA7RC1").unwrap();
        let result = unsafe { check_pass(pass.as_ptr(), password.as_ptr()) != 0 };

        assert!(!result);
    }
}

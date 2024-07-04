use crate::io_err;
use std::{
    fmt::{self, Display},
    io::{self, Read, Write},
};

fn read_file<R: Read>(mut file: R) -> io::Result<String> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Underlying implementation of get_config.
pub fn __get_userconfig<R: Read>(input: R) -> io::Result<UserConfig> {
    let config = read_file(input)?;

    let mut user = String::new();
    let mut mail = String::new();
    let mut log_path = String::new();
    let mut log_mode = String::new();

    // Helper closure to return an error.
    let err = || io_err!("Invalid userconfig file");

    let mut lines = config.lines();
    match lines.next() {
        Some("[gitconfig]") => (),
        _ => return Err(err()),
    }

    // Parse the config file.
    for config in lines {
        // Split the line in two.
        let line = config.split(" = ").collect::<Vec<&str>>();
        let key = line[0].trim();
        let val = line[1].trim();
        match key {
            "user" => user = val.to_string(),
            "mail" => mail = val.to_string(),
            "log_path" => log_path = val.to_string(),
            "log_mode" => log_mode = val.to_string(),
            _ => {}
        }
    }

    Ok(UserConfig {
        user,
        mail,
        log_path,
        log_mode,
    })
}

/// Underlying implementation of set_config.
pub fn __set_userconfig<W: Write>(
    user: &str,
    mail: &str,
    log_path: &str,
    log_mode: &str,
    mut out: W,
) -> io::Result<()> {
    let config = format!(
        "[gitconfig]\n\tuser = {}\n\tmail = {}\n\tlog_path = {}\n\tlog_mode = {}",
        user, mail, log_path, log_mode
    );
    out.write_all(config.as_bytes())?;
    Ok(())
}

/// Struct that represents the user's
/// configuration for commit purposes.
pub struct UserConfig {
    user: String,
    mail: String,
    log_path: String,
    log_mode: String,
}
impl UserConfig {
    pub fn get_mail(&self) -> &str {
        &self.mail
    }
    pub fn get_user(&self) -> &str {
        &self.user
    }
    pub fn get_log_path(&self) -> &str {
        &self.log_path
    }
    pub fn get_log_mode(&self) -> &str {
        &self.log_mode
    }
}

impl Display for UserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.user, self.mail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut v = vec![];
        __set_userconfig(
            "test_user",
            "test_mail",
            "/home/user/Documents/cmdlog.txt",
            "all",
            &mut v,
        )
        .unwrap();

        let config = __get_userconfig(&v[..]).unwrap();
        assert_eq!(config.get_user(), "test_user");
        assert_eq!(config.get_mail(), "test_mail");
        assert_eq!(config.get_log_path(), "/home/user/Documents/cmdlog.txt");
        assert_eq!(config.get_log_mode(), "all");
    }
}

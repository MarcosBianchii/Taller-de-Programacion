use crate::io_err;
use std::{
    fmt::{self, Display},
    io,
    str::{FromStr, Lines},
};

#[derive(Debug)]
pub enum ConfigEntry {
    Remote {
        name: String,
        url: String,
        fetch: String,
    },
    Branch {
        name: String,
        remote: String,
        merge: String,
    },
}

impl Display for ConfigEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();

        match self {
            ConfigEntry::Remote { name, url, fetch } => {
                s.push_str(&format!("[remote \"{name}\"]\n"));
                s.push_str(&format!("    url = {url}\n"));
                s.push_str(&format!("    fetch = {fetch}\n"));
            }
            ConfigEntry::Branch {
                name,
                remote,
                merge,
            } => {
                s.push_str(&format!("[branch \"{}\"]\n", name));
                s.push_str(&format!("    remote = {}\n", remote));
                s.push_str(&format!("    merge = {}\n", merge));
            }
        }

        write!(f, "{s}")
    }
}

fn branch_entry(name: String, lines: Lines) -> io::Result<ConfigEntry> {
    let mut remote = String::new();
    let mut merge = String::new();

    // Helper closure to return an error.
    let err = || io_err!("Invalid config file");

    for line in lines.filter(|l| !l.is_empty()) {
        if line.starts_with('[') || line.is_empty() {
            continue;
        }

        let line = line.split(" = ").collect::<Vec<&str>>();
        let key = line[0].trim();
        let val = line[1].trim();

        match key {
            "remote" => remote = val.to_string(),
            "merge" => merge = val.to_string(),
            _ => return Err(err()),
        }
    }

    Ok(ConfigEntry::Branch {
        name,
        remote,
        merge,
    })
}

fn remote_entry(name: String, lines: Lines) -> io::Result<ConfigEntry> {
    let mut url = String::new();
    let mut fetch = String::new();

    // Helper closure to return an error.
    let err = || io_err!("Invalid config file");

    for line in lines.filter(|l| !l.is_empty()) {
        if line.starts_with('[') {
            continue;
        }

        let line = line.split(" = ").collect::<Vec<&str>>();
        let key = line[0].trim();
        let val = line[1].trim();

        match key {
            "url" => url = val.to_string(),
            "fetch" => fetch = val.to_string(),
            _ => return Err(err()),
        }
    }

    Ok(ConfigEntry::Remote { name, url, fetch })
}

impl FromStr for ConfigEntry {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        // Helper closure to return an error.
        let err = || io_err!("Invalid config file");

        let (etype, name) = match lines.next() {
            None => return Err(err()),
            Some(header) => {
                let header = header.split(' ').collect::<Vec<&str>>();
                let etype = header[0];
                let name = header[1].replace(['"', ']'], "");

                (etype, name)
            }
        };

        match etype {
            "branch" => branch_entry(name, lines),
            "remote" => remote_entry(name, lines),
            _ => Err(err()),
        }
    }
}

impl ConfigEntry {
    pub fn name(&self) -> String {
        match self {
            ConfigEntry::Remote { name, .. } => name.to_owned(),
            ConfigEntry::Branch { name, .. } => name.to_owned(),
        }
    }

    pub fn is_remote(&self) -> bool {
        match self {
            ConfigEntry::Remote { .. } => true,
            ConfigEntry::Branch { .. } => false,
        }
    }

    pub fn new_branch(name: &str, remote: &str) -> Self {
        ConfigEntry::Branch {
            name: name.to_string(),
            remote: remote.to_string(),
            merge: format!("refs/heads/{}", name),
        }
    }
}

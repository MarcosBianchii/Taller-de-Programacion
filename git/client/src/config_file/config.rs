use super::config_entry::ConfigEntry;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[derive(Debug)]
pub struct Config {
    repositoryformatversion: u8,
    filemode: bool,
    bare: bool,
    logallrefupdates: bool,
    entries: HashMap<String, ConfigEntry>,
}

// Parses a given string to a given type returning an io::Result.
fn parse<T: FromStr>(val: &str) -> io::Result<T> {
    val.parse()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid config file"))
}

impl FromStr for Config {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("[core]") {
            return Err(io::Error::new(io::ErrorKind::Other, "Invalid config file"));
        }

        let mut reader = BufReader::new(s[1..].as_bytes());
        let mut buf = vec![];

        let mut repositoryformatversion = 0;
        let mut filemode = true;
        let mut bare = false;
        let mut logallrefupdates = true;

        // Get core section.
        reader.read_until(b'[', &mut buf)?;
        for line in buf.lines().flatten() {
            if let Some((key, val)) = line.split_once(" = ") {
                match key {
                    "repositoryformatversion" => repositoryformatversion = parse(val)?,
                    "filemode" => filemode = parse(val)?,
                    "bare" => bare = parse(val)?,
                    "logallrefupdates" => logallrefupdates = parse(val)?,
                    _ => {}
                }
            }
        }

        // Get entries section.
        buf.clear();
        let mut entries = HashMap::new();
        while let Ok(n) = reader.read_until(b'[', &mut buf) {
            if n == 0 {
                break;
            }

            let s = String::from_utf8_lossy(&buf).to_string();
            let entry = ConfigEntry::from_str(&s)?;
            entries.insert(entry.name(), entry);
            buf.clear();
        }

        Ok(Self {
            repositoryformatversion,
            filemode,
            bare,
            logallrefupdates,
            entries,
        })
    }
}

// Writes the config structure to the given output.
fn __write_to<W: Write>(config: Config, mut out: W) -> io::Result<()> {
    let mut s = String::from("[core]\n");
    s.push_str(&format!(
        "    repositoryformatversion = {}\n",
        config.repositoryformatversion
    ));

    s.push_str(&format!(
        "    filemode = {}\n",
        if config.filemode { "true" } else { "false" }
    ));

    s.push_str(&format!(
        "    bare = {}\n",
        if config.bare { "true" } else { "false" }
    ));

    s.push_str(&format!(
        "    logallrefupdates = {}\n",
        if config.logallrefupdates {
            "true"
        } else {
            "false"
        }
    ));

    for (_, entry) in config.entries.iter() {
        s.push_str(&entry.to_string());
    }

    write!(out, "{s}")
}

impl Deref for Config {
    type Target = HashMap<String, ConfigEntry>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl Config {
    pub fn read() -> io::Result<Config> {
        let config = fs::read_to_string(".git/config")?;
        Self::from_str(config.as_str())
    }

    pub fn write(self) -> io::Result<()> {
        let file = File::create(".git/config")?;
        __write_to(self, file)
    }

    pub fn add(&mut self, entry: ConfigEntry) {
        self.entries.insert(entry.name(), entry);
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use super::*;

    #[test]
    fn test() {
        let s = r#"[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
[remote "origin"]
    url = https://github.com/taller-1-fiuba-rust/23C2-Los-Krabby-Patty.git
    fetch = +refs/heads/*:refs/remotes/origin/*
[branch "main"]
    remote = origin
    merge = refs/heads/main
[branch "test-server_client"]
    remote = origin
    merge = refs/heads/test-server_client
[branch "parser"]
    remote = origin
    merge = refs/heads/parser
[branch "hash_object"]
    remote = origin
    merge = refs/heads/hash_object
[branch "server_client_protocol"]
    remote = origin
    merge = refs/heads/server_client_protocol
[branch "merge_init_hash"]
    remote = origin
    merge = refs/heads/merge_init_hash
[branch "core/library"]
    remote = origin
    merge = refs/heads/core/library
[branch "index"]
    remote = origin
    merge = refs/heads/index
[branch "command_parsing"]
    remote = origin
    merge = refs/heads/command_parsing
[branch "cmds"]
    remote = origin
    merge = refs/heads/cmds
[branch "cmd_status"]
    remote = origin
    merge = refs/heads/cmd_status
[branch "commit"]
    remote = origin
    merge = refs/heads/commit
[branch "cmds2"]
    remote = origin
    merge = refs/heads/cmds2
[branch "cmds3"]
    remote = origin
    merge = refs/heads/cmds3
[branch "status_fix"]
    remote = origin
    merge = refs/heads/status_fix
[branch "fetch"]
    remote = origin
    merge = refs/heads/fetch
"#;

        let config = Config::from_str(s).unwrap();
        let mut file = BufWriter::new(vec![]);
        __write_to(config, &mut file).unwrap();
        let out = String::from_utf8(file.into_inner().unwrap()).unwrap();
        println!("{out:#?}");
    }
}

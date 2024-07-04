use super::super::commands::*;
use crate::io_err;
use gtk::prelude::*;
use gtk::{Entry, ListBox};
use std::io;
use std::path::PathBuf;
use std::rc::Weak;
use utils::{log_err, log_ok};

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::changes_listbox::changes_listbox_refresh;
use crate::ui::principal_window::GitApp;

pub fn connect_add_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let add_listbox = strong_ref.changes_listbox.clone();

        strong_ref
            .add_button
            .connect_activate(move |_| add_button_handler(path_entry.clone(), add_listbox.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

fn get_files_in_dir_fmt(path: PathBuf, prefix: &str) -> io::Result<Vec<String>> {
    let mut entries = vec![];

    for entry in path.read_dir()?.flatten() {
        let path = entry.path();
        let file_name = match path.file_name() {
            Some(name) => name,
            None => return Err(io_err!("Error al obtener el nombre del archivo.")),
        };

        if file_name == ".git" {
            continue;
        }

        if path.is_dir() {
            entries.extend(get_files_in_dir_fmt(path, prefix)?);
        } else if let Ok(stripped) = path.strip_prefix(prefix) {
            entries.push(stripped.to_string_lossy().to_string());
        } else {
            entries.push(path.to_string_lossy().to_string());
        }
    }

    Ok(entries)
}

fn get_files_in_dir(path: PathBuf) -> io::Result<Vec<String>> {
    let path = path.canonicalize()?;
    let prefix = std::env::current_dir()?
        .to_owned()
        .to_string_lossy()
        .to_string();

    get_files_in_dir_fmt(path, &prefix)
}

pub fn parse_entry_for_add_rm(user_text: &str) -> io::Result<Vec<String>> {
    let split = user_text.split_whitespace();
    let files: Vec<_> = split.map(|s| PathBuf::from(s.trim())).collect();

    let mut files_to_add = vec![];
    for file in files {
        if file.is_dir() {
            match get_files_in_dir(file) {
                Ok(mut files) => {
                    files_to_add.append(&mut files);
                }

                Err(err) => {
                    log_err!("add", err);
                    eprintln!("Error al agregar archivos: {}", err);
                    return Err(io_err!("Error al agregar archivos"));
                }
            }
        } else {
            let path = file.to_string_lossy().to_string();
            files_to_add.push(path);
        }
    }

    Ok(files_to_add)
}

pub fn add_button_handler(add_entry: Entry, add_listbox: ListBox) {
    // Get the files from the entry.
    let text = add_entry.text();
    let files_to_add = match parse_entry_for_add_rm(&text) {
        Ok(files) => files,
        Err(err) => {
            log_err!("add", err);
            eprintln!("Error al agregar archivos: {}", err);
            return;
        }
    };

    match add(files_to_add) {
        Ok(()) => {
            log_ok!("add");
            println!("Archivos agregados exitosamente.");
        }
        Err(err) => {
            log_err!("add", err);
            eprintln!("Error al agregar archivos: {}", err);
        }
    };

    add_entry.set_text("");
    changes_listbox_refresh(add_listbox);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn add_dir() {
        let path = PathBuf::from(".");

        match get_files_in_dir(path) {
            Ok(files) => {
                println!("Files: {:#?}", files);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

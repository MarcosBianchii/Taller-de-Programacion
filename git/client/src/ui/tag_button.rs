use super::super::commands::*;
use crate::io_err;
use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;
use crate::ui::terminal_text_view::shows_in_terminal;
use gtk::prelude::*;
use gtk::{Entry, TextView};
use std::io;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

pub fn connect_tag_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let terminal_text_view = strong_ref.terminal_text_view.clone();

        strong_ref.tag_button.connect_activate(move |_| {
            tag_button_handler(path_entry.clone(), terminal_text_view.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn tag_button_handler(path_entry: Entry, terminal_text_view: TextView) {
    let entry = path_entry.text().to_string();
    let entries: Vec<&str> = entry.split_whitespace().collect();

    // add/add-force nombre 124123123123

    let action = if entry.is_empty() {
        TagCommand::List
    } else if entries.len() >= 2 {
        let hash = if entries.len() >= 3 {
            Some(entries[2].to_string())
        } else {
            None
        };

        match entries[0] {
            "add" => TagCommand::Add {
                name: entries[1].to_string(),
                hash,
                msg: None,
            },

            "add-force" => TagCommand::AddForce {
                name: entries[1].to_string(),
                hash,
                msg: None,
            },

            "delete" => TagCommand::Del {
                name: entries[1].to_string(),
            },
            _ => {
                log_err!("tag", io_err!("TagCommand incorrecto"));
                eprintln!("Error al hacer tag {}", io_err!("TagCommand incorrecto"));
                return;
            }
        }
    } else {
        log_err!("tag", io_err!("Cantidad de parametros incorrecta"));
        eprintln!(
            "Error al hacer tag {}",
            io_err!("Cantidad de parametros incorrecta")
        );
        return;
    };

    match tag(action) {
        Ok(None) => {
            log_ok!("tag");
            println!("Tag realizado correctamente")
        }
        Ok(Some(tags)) => {
            shows_in_terminal(tags, terminal_text_view);

            log_ok!("tag");
            println!("Tag realizado correctamente")
        }
        Err(err) => {
            log_err!("tag", err);
            eprintln!("Error al hacer tag: {}", err);
        }
    }

    path_entry.set_text("");
}

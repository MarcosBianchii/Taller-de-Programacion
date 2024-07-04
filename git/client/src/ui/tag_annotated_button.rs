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

pub fn connect_tag_annotated_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let terminal_text_view = strong_ref.terminal_text_view.clone();

        strong_ref.tag_annotated_button.connect_activate(move |_| {
            tag_annotated_button_handler(path_entry.clone(), terminal_text_view.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn tag_annotated_button_handler(path_entry: Entry, terminal_text_view: TextView) {
    let entry = path_entry.text().to_string();
    let entries: Vec<&str> = entry.split_whitespace().collect();

    let action = if entry.is_empty() {
        TagCommand::List
    } else if entries.len() >= 3 {
        let hash = if entries.len() >= 4 {
            Some(entries[3].to_string())
        } else {
            None
        };

        match entries[0] {
            "add" => TagCommand::Add {
                name: entries[1].to_string(),
                hash,
                msg: Some(entries[2].to_string()),
            },

            "add-force" => TagCommand::AddForce {
                name: entries[1].to_string(),
                hash,
                msg: Some(entries[2].to_string()),
            },

            _ => {
                log_err!("tag-annotated", io_err!("TagCommand incorrecto"));
                eprintln!("Error al hacer tag {}", io_err!("TagCommand incorrecto"));
                return;
            }
        }
    } else if entries[0] == "delete" {
        TagCommand::Del {
            name: entries[1].to_string(),
        }
    } else {
        log_err!(
            "tag-annotated",
            io_err!("Cantidad de parametros incorrecta")
        );
        eprintln!(
            "Error al hacer tag-annotated {}",
            io_err!("Cantidad de parametros incorrecta")
        );
        return;
    };

    match tag(action) {
        Ok(None) => {
            log_ok!("tag-annotated");
            println!("Tag-Annotated realizado correctamente")
        }
        Ok(Some(tags)) => {
            shows_in_terminal(tags, terminal_text_view);

            log_ok!("tag-annotated");
            println!("Tag-Annotated realizado correctamente")
        }
        Err(err) => {
            log_err!("tag-annotated", err);
            eprintln!("Error al hacer tag-annotated: {}", err);
        }
    }

    path_entry.set_text("");
}

use super::super::commands::*;
use gtk::prelude::*;
use gtk::{Entry, TextView};
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;
use crate::ui::terminal_text_view::shows_in_terminal;

pub fn connect_remote_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let terminal_text_view = strong_ref.terminal_text_view.clone();
        strong_ref.remote_button.connect_activate(move |_| {
            remote_button_handler(path_entry.clone(), terminal_text_view.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn remote_button_handler(remote_entry: Entry, terminal_text_view: TextView) {
    let entry = remote_entry.text().to_string();

    let entries: Vec<&str> = entry.split_whitespace().collect();

    let action = if entry.is_empty() {
        RemoteCommand::List
    } else {
        match entries[0] {
            "add" => {
                if entries.len() != 3 {
                    println!("Cantidad erronea de parametros");
                    return;
                }
                RemoteCommand::Add {
                    name: entries[1].to_string(),
                    url: entries[2].to_string(),
                }
            }
            "remove" => {
                if entries.len() != 2 {
                    println!("Cantidad erronea de parametros");
                    return;
                }
                RemoteCommand::Rem {
                    name: entries[1].to_string(),
                }
            }
            _ => {
                if entry.is_empty() {
                    RemoteCommand::List
                } else {
                    if let Err(err) = log_command(
                        "remote".to_string(),
                        LogMsgStatus::ErrOnExecution("Invalid parameters".to_string()),
                    ) {
                        println!("Logging failed with error : {}", err);
                    }
                    println!("Comando invalido");
                    return;
                }
            }
        }
    };

    match remote(action) {
        Ok(Some(result)) => {
            log_ok!("remote");
            shows_in_terminal(result, terminal_text_view);
        }

        Err(err) => {
            log_err!("remote", err);
            eprintln!("Error al ejecutar clone: {}", err);
        }
        _ => log_ok!("remote"),
    };

    remote_entry.set_text("");
}

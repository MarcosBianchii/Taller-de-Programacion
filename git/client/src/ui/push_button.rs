use crate::commands::{push, PushCommand};
use crate::logging::{log_command, LogMsgStatus};
use gtk::prelude::*;
use gtk::Entry;
use std::rc::Weak;
use utils::{log_err, log_ok};

use crate::ui::principal_window::GitApp;

pub fn connect_push_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        strong_ref
            .push_button
            .connect_activate(move |_| push_button_handler(path_entry.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn push_button_handler(push_entry: Entry) {
    let entry = push_entry.text().to_string();

    let entries: Vec<&str> = entry.split_whitespace().collect();

    let action = if entry.is_empty() {
        PushCommand::Push
    } else {
        if entries.len() != 2 {
            println!("Cantidad erronea de parametros");
            return;
        }

        PushCommand::SetUpstream {
            remote: entries[0].to_string(),
            branch: entries[1].to_string(),
        }
    };

    match push(action) {
        Ok(()) => {
            log_ok!("push");
            println!("Push realizado correctamente");
        }
        Err(err) => {
            log_err!("push", err);
            eprintln!("Error al hacer push: {}", err);
        }
    };

    push_entry.set_text("");
}

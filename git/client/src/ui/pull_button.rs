use super::super::commands::*;
use crate::ui::changes_listbox::changes_listbox_refresh;
use gtk::prelude::*;
use gtk::{Entry, ListBox};
use std::rc::Weak;
use utils::{log_err, log_ok};

use crate::logging::{log_command, LogMsgStatus};
use crate::ui::principal_window::GitApp;

pub fn connect_pull_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let add_listbox = strong_ref.changes_listbox.clone();

        strong_ref.pull_button.connect_activate(move |_| {
            pull_button_handler(path_entry.clone(), add_listbox.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn pull_button_handler(path_entry: Entry, add_listbox: ListBox) {
    match pull(path_entry.text().to_string().as_str()) {
        Ok(()) => {
            log_ok!("pull");
            println!("Pull realizado correctamente.");
        }
        Err(err) => {
            log_err!("pull", err);
            eprintln!("Error al hacer pull: {}", err);
        }
    };

    path_entry.set_text("");
    changes_listbox_refresh(add_listbox);
}

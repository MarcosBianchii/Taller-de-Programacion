use super::super::commands::*;
use gtk::prelude::*;
use gtk::Entry;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;

pub fn connect_rebase_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        strong_ref
            .rebase_button
            .connect_activate(move |_| rebase_button_handler(path_entry.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn rebase_button_handler(path_entry: Entry) {
    match rebase(path_entry.text().to_string().as_str()) {
        Ok(()) => {
            log_ok!("rebase");
            println!("Rebase realizado correctamente")
        }
        Err(err) => {
            log_err!("rebase", err);
            eprintln!("Error al hacer rebase: {}", err);
        }
    }

    path_entry.set_text("");
}

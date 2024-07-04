use super::super::commands::*;
use gtk::prelude::*;
use gtk::Entry;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;

pub fn connect_merge_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();

        strong_ref
            .merge_button
            .connect_activate(move |_| merge_button_handler(path_entry.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn merge_button_handler(merge_entry: Entry) {
    let message = merge_entry.text().to_string();
    if !message.is_empty() {
        match merge(message.as_str()) {
            Ok(()) => {
                log_ok!("merge");
                println!("Merge realizado correctamente");
            }
            Err(err) => {
                log_err!("merge", err);
                eprintln!("Error al ejecutar merge: {}", err);
            }
        };

        merge_entry.set_text("");
    }
}

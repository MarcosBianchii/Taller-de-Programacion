use super::super::commands::*;
use gtk::prelude::*;
use gtk::Entry;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;

pub fn connect_fetch_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        strong_ref
            .fetch_button
            .connect_activate(move |_| fetch_button_handler(path_entry.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn fetch_button_handler(fetch_entry: Entry) {
    match fetch(fetch_entry.text().to_string().as_str()) {
        Ok(()) => {
            log_ok!("fetch");
            println!(
                "Fetch desde: {} ,realizado correctamente",
                fetch_entry.text().to_string().as_str()
            );
        }
        Err(err) => {
            log_err!("fetch", err);
            eprintln!("Error al hacer fetch: {}", err);
        }
    };

    fetch_entry.set_text("");
}

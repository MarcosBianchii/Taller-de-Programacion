use super::super::commands::*;
use super::add_button::parse_entry_for_add_rm;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::changes_listbox::changes_listbox_refresh;
use crate::ui::principal_window::GitApp;
use gtk::prelude::*;
use gtk::{Entry, ListBox};
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

pub fn connect_remove_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let add_listbox = strong_ref.changes_listbox.clone();
        strong_ref.remove_button.connect_activate(move |_| {
            remove_button_handler(path_entry.clone(), add_listbox.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn remove_button_handler(remove_entry: Entry, add_listbox: ListBox) {
    let text = remove_entry.text();
    let files_to_remove = match parse_entry_for_add_rm(&text) {
        Ok(files) => files,
        Err(err) => {
            log_err!("rm", err);
            eprintln!("Error al parsear la entrada: {}", err);
            return;
        }
    };

    for file in &files_to_remove {
        println!("Archivo a remover: {}", file);
    }

    match rm(files_to_remove) {
        Ok(()) => {
            log_ok!("rm");
            println!("Archivos removidos exitosamente.");
        }
        Err(err) => {
            log_err!("rm", err);
            eprintln!("Error al remover archivos: {}", err);
        }
    };

    remove_entry.set_text("");
    changes_listbox_refresh(add_listbox);
}

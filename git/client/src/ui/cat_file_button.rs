use super::super::commands::*;
use gtk::prelude::*;
use gtk::{Entry, TextView};
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;
use crate::ui::terminal_text_view::show_in_terminal;

pub fn connect_cat_file_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let terminal_text_view = strong_ref.terminal_text_view.clone();
        strong_ref.cat_file_button.connect_activate(move |_| {
            cat_file_button_handler(path_entry.clone(), terminal_text_view.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn cat_file_button_handler(cat_file_entry: Entry, terminal_text_view: TextView) {
    if !cat_file_entry.text().is_empty() {
        match cat_file(cat_file_entry.text().to_string().as_str()) {
            Ok(content) => {
                log_ok!("cat-file");
                show_in_terminal(content.2, terminal_text_view);
            }
            Err(err) => {
                log_err!("cat-file", err);
                eprintln!("Error al ejecutar cat-file: {}", err);
            }
        };
    }

    cat_file_entry.set_text("");
}

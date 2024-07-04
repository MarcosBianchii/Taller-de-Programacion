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

pub fn connect_ls_tree_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let terminal_text_view = strong_ref.terminal_text_view.clone();

        strong_ref.ls_tree_button.connect_activate(move |_| {
            ls_tree_button_handler(path_entry.clone(), terminal_text_view.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn ls_tree_button_handler(ls_tree_entry: Entry, terminal_text_view: TextView) {
    match ls_tree(ls_tree_entry.text().to_string().as_str()) {
        Ok(content) => {
            log_ok!("ls-tree");

            let content = content.replace('\0', "");
            show_in_terminal(content.to_string(), terminal_text_view);
            println!("Ls tree ejectuado correctamente: {}", content);
        }
        Err(err) => {
            log_err!("ls-tree", err);
            eprintln!("Error al ejecutar ls-tree: {}", err);
        }
    };

    ls_tree_entry.set_text("");
}

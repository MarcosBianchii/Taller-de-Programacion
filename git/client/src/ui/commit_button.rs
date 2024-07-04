use super::super::commands::*;
use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::changes_listbox::changes_listbox_refresh;
use crate::ui::principal_window::GitApp;
use crate::ui::terminal_text_view::show_in_terminal;
use gtk::prelude::*;
use gtk::{Entry, ListBox, TextView};
use std::io;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

pub fn connect_commit_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let commit_entry = strong_ref.commit_entry.clone();
        let add_listbox = strong_ref.changes_listbox.clone();
        let terminal_text_view = strong_ref.terminal_text_view.clone();

        strong_ref.commit_button.connect_clicked(move |_| {
            if let Ok(()) = commit_button_handler(
                commit_entry.clone(),
                add_listbox.clone(),
                terminal_text_view.clone(),
            ) {
            } else {
                println!("Error al conectar botón de commit");
            }
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn commit_button_handler(
    commit_entry: Entry,
    add_listbox: ListBox,
    terminal_text_view: TextView,
) -> io::Result<()> {
    let message = commit_entry.text().to_string();

    if !message.is_empty() {
        match commit(message.as_str()) {
            Ok(hash) => {
                log_ok!("commit");
                show_in_terminal(format!("Commit realizado: {hash}"), terminal_text_view);
            }
            Err(err) => {
                log_err!("commit", err);
                eprintln!("Error al ejecutar commit: {}", err);
            }
        };

        commit_entry.set_text("");
        changes_listbox_refresh(add_listbox);
    }
    Ok(())
}

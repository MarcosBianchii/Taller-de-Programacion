use gtk::prelude::*;
use gtk::{Button, Entry};
use std::rc::Weak;

use crate::ui::principal_window::GitApp;

pub fn connect_commit_message_entry(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let commit_entry = strong_ref.commit_entry.clone();
        let commit_button = strong_ref.commit_button.clone();

        strong_ref.commit_entry.connect_changed(move |_| {
            commit_message_handler(commit_entry.clone(), commit_button.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn commit_message_handler(commit_entry: Entry, commit_button: Button) {
    let entry_text = commit_entry.text().to_string();
    let is_empty = entry_text.is_empty();

    commit_button.set_sensitive(!is_empty);
}

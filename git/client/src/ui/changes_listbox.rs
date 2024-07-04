use crate::commands::{status, FileStatus};
use crate::logging::{log_command, LogMsgStatus};
use gtk::prelude::*;
use gtk::{Entry, ListBox, ListBoxRow};
use std::rc::Weak;
use utils::{log_err, log_ok};

use crate::ui::principal_window::GitApp;

pub fn connect_changes_listbox(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();

        strong_ref
            .changes_listbox
            .connect_row_selected(move |_, row| changes_listbox_handler(row, path_entry.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn changes_listbox_handler(row: Option<&ListBoxRow>, path_entry: Entry) {
    if let Some(selected_row) = row {
        if let Some(label) = selected_row
            .child()
            .and_then(|label| label.downcast::<gtk::Label>().ok())
        {
            let label_text = label.text().as_str().to_string();
            let file_name = remove_status(label_text.as_str());
            path_entry.set_text(file_name);
        }
    }
}

pub fn changes_listbox_refresh(changes_listbox: ListBox) {
    for list_row in changes_listbox.children() {
        changes_listbox.remove(&list_row);
    }

    let statuses = match status() {
        Ok(statuses) => {
            log_ok!("status");
            statuses
        }
        Err(err_msg) => {
            log_err!("status", err_msg);
            return;
        }
    };

    for file_status in statuses {
        let list_row = gtk::ListBoxRow::new();
        let status_string = file_status_to_string(&file_status);
        let label = gtk::Label::new(Some(&status_string));
        list_row.add(&label);
        changes_listbox.add(&list_row);
        list_row.show_all();
    }
}

fn remove_status(input: &str) -> &str {
    if input.starts_with("New: ") {
        match input.strip_prefix("New: ") {
            Some(s) => s,
            None => input,
        }
    } else if input.starts_with("Staged: ") {
        match input.strip_prefix("Staged: ") {
            Some(s) => s,
            None => input,
        }
    } else if input.starts_with("Modified: ") {
        match input.strip_prefix("Modified: ") {
            Some(s) => s,
            None => input,
        }
    } else if input.starts_with("Deleted: ") {
        match input.strip_prefix("Deleted: ") {
            Some(s) => s,
            None => input,
        }
    } else {
        input
    }
}

pub fn file_status_to_string(file_status: &FileStatus) -> String {
    match file_status {
        FileStatus::New(filename) => format!("New: {}", filename),
        FileStatus::Staged(filename) => format!("Staged: {}", filename),
        FileStatus::Modified(filename) => format!("Modified: {}", filename),
        FileStatus::Deleted(filename) => format!("Deleted: {}", filename),
    }
}

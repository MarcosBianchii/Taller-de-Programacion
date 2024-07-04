use crate::ui::changes_listbox::changes_listbox_refresh;
use gtk::prelude::*;
use gtk::ListBox;
use std::rc::Weak;

use crate::ui::principal_window::GitApp;

pub fn connect_status_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let add_listbox = strong_ref.changes_listbox.clone();
        strong_ref
            .status_button
            .connect_activate(move |_| status_button_handler(add_listbox.clone()));
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn status_button_handler(add_listbox: ListBox) {
    changes_listbox_refresh(add_listbox);
}

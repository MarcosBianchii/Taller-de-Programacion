use gtk::prelude::*;
use gtk::ApplicationWindow;
use std::rc::{Rc, Weak};

use crate::ui::keep_or_remove_conflict::keep_or_remove_window::{
    KeepOrRemoveResult, KeepRemoveWindow,
};

pub fn connect_keep_button(keep_window: &Weak<KeepRemoveWindow>) {
    if let Some(strong_ref) = keep_window.upgrade() {
        let application_window = strong_ref.application_window.clone();
        let strong_ref_clone = Rc::clone(&strong_ref);

        strong_ref.keep_button.connect_clicked(move |_| {
            keep_button_handler(application_window.clone(), strong_ref_clone.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn keep_button_handler(
    application_window: ApplicationWindow,
    keep_remove_window: Rc<KeepRemoveWindow>,
) {
    keep_remove_window.update_custom_state(KeepOrRemoveResult::Keep);
    application_window.close();
    gtk::main_quit();
}

use super::super::commands::*;
use gtk::prelude::*;
use gtk::{Entry, TextView};
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::current_branch_text_view::show_current_branch;
use crate::ui::principal_window::GitApp;

pub fn connect_checkout_button(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        let current_branch_text_view = strong_ref.current_branch_text_view.clone();
        strong_ref.checkout_button.connect_activate(move |_| {
            checkout_button_handler(path_entry.clone(), current_branch_text_view.clone())
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn checkout_button_handler(checkout_entry: Entry, current_branch_text_view: TextView) {
    let branch = checkout_entry.text().to_string();
    if branch.is_empty() {
        eprintln!("Error: no se ha ingresado un nombre de rama.");
        return;
    }
    match checkout(branch.as_str()) {
        Ok(()) => {
            log_ok!("checkout");
            show_current_branch(branch, current_branch_text_view);
            println!("Checkout realizado correctamente");
        }
        Err(err) => {
            log_err!("checkout", err);
            eprintln!("Error al ejecutar checkout: {}", err);
        }
    };

    checkout_entry.set_text("");
}

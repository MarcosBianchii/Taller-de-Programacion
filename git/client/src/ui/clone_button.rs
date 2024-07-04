use super::super::commands::*;
use crate::logging::log_command;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;
use crate::ui::principal_window::UiEvent;
use gtk::glib;
use gtk::prelude::*;
use gtk::Entry;
use std::io;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

pub fn connect_clone_button(git_app: &Weak<GitApp>, sender: glib::Sender<UiEvent>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        strong_ref.clone_button.connect_activate(move |_| {
            if let Ok(()) = clone_button_handler(path_entry.clone(), sender.clone()) {
            } else {
                println!("Error al conectar botón de clone");
            }
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn clone_button_handler(path_entry: Entry, sender: glib::Sender<UiEvent>) -> io::Result<()> {
    let result = clone(path_entry.text().to_string().as_str());
    match result {
        Ok(()) => {
            println!("Clonado correctamente.");
            log_ok!("clone");
            sender
                .send(UiEvent::Clone)
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        }
        Err(err) => {
            log_err!("clone", err);
            eprintln!("Error al hacer clone: {}", err);
        }
    }
    path_entry.set_text("");
    Ok(())
}

use super::super::commands::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::Entry;
use std::rc::Weak;
use utils::log_err;
use utils::log_ok;

use crate::logging::log_command;

//use std::sync::mpsc::Sender;

use crate::io_err;
use crate::logging::LogMsgStatus;
use crate::ui::principal_window::GitApp;
use crate::ui::principal_window::UiEvent;
use std::io;

pub fn connect_branch_button(git_app: &Weak<GitApp>, transmiter: glib::Sender<UiEvent>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let path_entry = strong_ref.path_entry.clone();
        strong_ref.branch_button.connect_activate(move |_| {
            if let Ok(()) = branch_button_handler(path_entry.clone(), transmiter.clone()) {
            } else {
                println!("Error al conectar botón de branch");
            }
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn branch_button_handler(
    branch_entry: Entry,
    transmiter: glib::Sender<UiEvent>,
) -> io::Result<()> {
    let branch_name = branch_entry.text().to_string();
    if branch_name.is_empty() {
        // Obtain all local branches an send
        let result = branch(None);
        match result {
            Ok(Some(_)) => {
                log_ok!("branch");
                transmiter
                    .send(UiEvent::Branch)
                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
            }
            Err(err) => {
                log_err!("branch", err);
                eprintln!("Error al actualizar branches: {}", err);
            }
            Ok(None) => {
                log_err!("branch", io_err!("Error inesperado: unreacheble"));
            }
        }

        return Ok(());
    }

    match branch(Some(branch_name.to_string())) {
        Ok(None) => {
            transmiter
                .send(UiEvent::NewBranch(branch_name))
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
            println!("Branch creada correctamente.");
            log_ok!("branch");
        }
        Err(err) => {
            log_err!("branch", err);
            eprintln!("Error al ejecutar branch: {}", err);
        }
        _ => {}
    }

    branch_entry.set_text("");
    Ok(())
}

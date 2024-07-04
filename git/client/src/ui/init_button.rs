use super::super::commands::*;
use crate::ui::principal_window::UiEvent;
use crate::{
    logging::{log_command, LogMsgStatus},
    ui::principal_window::GitApp,
};
use gtk::glib;
use gtk::prelude::*;
use std::io;
use std::rc::Weak;
use utils::{log_err, log_ok};

pub fn connect_init_button(git_app: &Weak<GitApp>, sender: glib::Sender<UiEvent>) {
    if let Some(strong_ref) = git_app.upgrade() {
        strong_ref.init_button.connect_activate(move |_| {
            if let Ok(()) = init_button_handler(sender.clone()) {
            } else {
                println!("Error al conectar botón de init");
            }
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn init_button_handler(sender: glib::Sender<UiEvent>) -> io::Result<()> {
    match init(".") {
        Ok(_) => {
            log_ok!("init");
            println!("Init ejecutado correctamente");
            sender
                .send(UiEvent::Init)
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        }
        Err(err) => {
            log_err!("init", err);
            eprintln!("Error al realizar init: {}", err);
        }
    }
    Ok(())
}

use crate::io_err;
use crate::ui::keep_or_remove_conflict::{delete_button, keep_button};
use gtk::prelude::*;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

pub struct KeepRemoveWindow {
    pub(crate) application_window: gtk::ApplicationWindow,
    pub(crate) keep_button: gtk::Button,
    pub(crate) delete_button: gtk::Button,
    pub(crate) path_text_view: gtk::TextView,
    pub(crate) state: RefCell<KeepOrRemoveResult>,
}

#[derive(Clone)]
pub enum KeepOrRemoveResult {
    Keep,
    Delete,
    Error,
}

impl KeepRemoveWindow {
    pub fn new(path_to_file: String) -> io::Result<Rc<KeepRemoveWindow>> {
        if gtk::init().is_err() {
            return Err(io_err!("Failed to initialize GTK"));
        }

        let keep_or_remove_window: &str = include_str!("../resources/keep_or_remove_window.ui");
        let builder: gtk::Builder = gtk::Builder::from_string(keep_or_remove_window);

        // ApplicationWindow
        let application_window: gtk::ApplicationWindow = builder
            .object("main_window")
            .ok_or(io_err!("Error al obtener main_window"))?;

        // Keep Button
        let keep_button: gtk::Button = builder
            .object("keep_button")
            .ok_or(io_err!("Error al obtener keep_button"))?;

        // Delete Button
        let delete_button: gtk::Button = builder
            .object("delete_button")
            .ok_or(io_err!("Error al obtener delete_button"))?;

        // Path Text View
        let path_text_view: gtk::TextView = builder
            .object("path_text_view")
            .ok_or(io_err!("Error al obtener path_text_view"))?;

        let principal_window = KeepRemoveWindow {
            application_window,
            keep_button,
            delete_button,
            path_text_view,
            state: RefCell::new(KeepOrRemoveResult::Error),
        };

        principal_window
            .application_window
            .set_title("Keep or Remove Conflict");
        principal_window.application_window.show_all();

        let principal_window = Rc::new(principal_window);
        let weak_ref = Rc::downgrade(&principal_window);

        // Set text to TextView
        let buffer = principal_window
            .path_text_view
            .buffer()
            .ok_or(io_err!("Error al obtener buffer de path_text_view"))?;

        buffer.set_text(&path_to_file);

        // Keep button handler
        keep_button::connect_keep_button(&weak_ref);

        // Delete button handler
        delete_button::connect_delete_button(&weak_ref);

        Ok(principal_window)
    }
    pub fn run(&self) {
        gtk::main();
    }

    // Method to update the custom state
    pub fn update_custom_state(&self, new_state: KeepOrRemoveResult) {
        *self.state.borrow_mut() = new_state;
    }

    // Method to get the custom state
    pub fn get_custom_state(&self) -> KeepOrRemoveResult {
        self.state.borrow().clone()
    }
}

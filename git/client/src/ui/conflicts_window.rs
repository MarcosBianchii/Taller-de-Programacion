use crate::io_err;
use gtk::prelude::*;
use std::{
    fs::File,
    io::{self, Write},
};

pub struct GtkConflict {
    window: gtk::Window,
}

impl GtkConflict {
    pub fn new(conflict_file: String, path_to_file: String) -> io::Result<Self> {
        if gtk::init().is_err() {
            return Err(io_err!("Failed to initialize GTK"));
        }

        let git_glade_src: &str = include_str!("resources/conflict_window.ui");
        let builder: gtk::Builder = gtk::Builder::from_string(git_glade_src);

        // Window
        let window: gtk::Window = builder
            .object("main_window")
            .ok_or(io_err!("Window not found"))?;

        window.set_title("Git Conflict - 3-Way-Merge");

        let done_button: gtk::Button = builder
            .object("done_button")
            .ok_or(io_err!("Done button not found"))?;

        // Use text view to show the conflicts
        let text_view: gtk::TextView = builder
            .object("text_view")
            .ok_or(io_err!("Text view not found"))?;

        // Set the conflict text in the text view
        let buffer = text_view.buffer().ok_or(io_err!("Buffer not found"))?;
        buffer.set_text(&conflict_file);

        let window_moved = window.clone();
        done_button.connect_clicked(move |_| {
            let buffer = match text_view.buffer() {
                Some(buffer) => buffer,
                None => {
                    println!("Error getting text from buffer");
                    return;
                }
            };

            let bytes = match buffer.text(&buffer.start_iter(), &buffer.end_iter(), false) {
                Some(bytes) => bytes,
                None => {
                    println!("Error getting text from buffer");
                    return;
                }
            };

            if let Ok(mut file) = File::create(&path_to_file) {
                match file.write_all(bytes.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error writing file: {e}");
                        return;
                    }
                }
            }

            window_moved.close();
        });

        Ok(GtkConflict { window })
    }

    pub fn run(&self) {
        self.window.show_all();
        let window = self.window.clone();
        window.connect_destroy(|_| {
            gtk::main_quit();
        });

        gtk::main();
    }
}

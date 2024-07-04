use gtk::prelude::*;
use std::env;
use std::string::ToString;

const APP_TITLE: &str = "Git Client";
pub fn initialize_window(application_window: &gtk::ApplicationWindow) {
    let mut title = String::new();

    if let Ok(current_dir) = env::current_dir() {
        let current_dir_str = current_dir.to_string_lossy().to_string();
        title.push_str(&current_dir_str);
    }

    // Title example: /Path/where/client/is/executed - Git Client
    title.push_str(" - ");
    title.push_str(APP_TITLE);

    application_window.set_title(&title);

    // Show ApplicationWindow
    application_window.show_all();

    // Exit button
    application_window.connect_destroy(|_| {
        gtk::main_quit();
    });
}

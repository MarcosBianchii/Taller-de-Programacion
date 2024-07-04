use client::ui::principal_window;
use std::rc::Rc;
fn main() {
    if let Some(window) = principal_window::GitApp::new() {
        let main_app = Rc::new(window);
        main_app.run();
    }
}

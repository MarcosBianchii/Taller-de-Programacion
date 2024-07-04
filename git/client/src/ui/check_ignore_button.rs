// use gtk::prelude::*;
// use std::rc::Weak;
//
// use crate::ui::principal_window::GitApp;
//
// pub fn _connect_check_ignore_button(git_app: &Weak<GitApp>) {
//     if let Some(strong_ref) = git_app.upgrade() {
//         strong_ref
//             ._check_ignore_button
//             .connect_activate(move |_| _check_ignore_button_handler());
//     } else {
//         println!("Error al hacer upgrade");
//     }
// }
//
// pub fn _check_ignore_button_handler() {
//     // check_ignore()
// }

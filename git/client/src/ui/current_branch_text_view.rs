use gtk::prelude::{TextBufferExt, TextViewExt};
use gtk::TextView;

pub fn show_current_branch(output: String, current_branch_text_view: TextView) {
    if let Some(buffer) = current_branch_text_view.buffer() {
        buffer.set_text(output.as_str());
    }
}

use gtk::prelude::{TextBufferExt, TextViewExt};
use gtk::TextView;

pub fn show_in_terminal(output: String, terminal_text_view: TextView) {
    if let Some(buffer) = terminal_text_view.buffer() {
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, &output);
        buffer.insert(&mut end, "\n");
    }
}

pub fn shows_in_terminal(outputs: Vec<String>, terminal_text_view: TextView) {
    if let Some(buffer) = terminal_text_view.buffer() {
        let mut end = buffer.end_iter();

        for output in outputs {
            buffer.insert(&mut end, &output);
        }
        buffer.insert(&mut end, "\n");
    }
}

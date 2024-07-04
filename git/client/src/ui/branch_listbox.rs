use crate::plumbing::log::__log;
use crate::plumbing::refs::{get_branch_ref, get_local_branches};
use crate::ui::history_listbox::set_list_box_row;
use crate::ui::principal_window::GitApp;
use gtk::glib::Cast;
use gtk::prelude::{BinExt, ContainerExt, LabelExt, ListBoxExt, WidgetExt};
use gtk::{ListBox, ListBoxRow};
use std::collections::HashSet;
use std::io;
use std::rc::Weak;

// init branches listbox with local branches
pub fn branches_listbox_init(branches_listbox: ListBox) -> io::Result<()> {
    // obtain all branches
    let local_branches = get_local_branches()?;
    local_branches.iter().for_each(|elemento| {
        let list_row = gtk::ListBoxRow::new();
        let label = gtk::Label::new(Some(elemento));
        list_row.add(&label);
        branches_listbox.add(&list_row);
        list_row.show_all();
    });

    Ok(())
}

// add new branch to the branches listbox
pub fn add_new_branch(branches_listbox: ListBox, branch_name: String) -> io::Result<()> {
    let list_row = gtk::ListBoxRow::new();
    let label = gtk::Label::new(Some(&branch_name));
    list_row.add(&label);
    branches_listbox.add(&list_row);
    list_row.show_all();
    Ok(())
}

// get specified branch commit history using log command
fn get_commit_history(branch_name: String) -> io::Result<Vec<String>> {
    let hash_commit = get_branch_ref(&branch_name)?;
    __log(&hash_commit, &mut HashSet::new())
}

// connect branches listbox
pub fn connect_branch_listbox(git_app: &Weak<GitApp>) {
    // get history listbox
    if let Some(strong_ref) = git_app.upgrade() {
        let history_listbox = strong_ref.history_listbox.clone();
        strong_ref
            .branches_listbox
            .connect_row_selected(move |_, row| {
                branch_listbox_handler(row, history_listbox.clone())
            });
    } else {
        println!("Error al hacer upgrade");
    }
}

// branch listbox handler
pub fn branch_listbox_handler(row: Option<&ListBoxRow>, history_list_box: ListBox) {
    if let Some(selected_row) = row {
        if let Some(label) = selected_row
            .child()
            .and_then(|label| label.downcast::<gtk::Label>().ok())
        {
            let branch_name = label.text().as_str().to_string();
            println!("Selected branch: {}", branch_name);
            match refresh_history_list_box(branch_name, history_list_box) {
                Ok(_) => println!("History listbox refreshed"),
                Err(_) => println!("Error al refrescar el history listbox"),
            }
        }
    }
}

// refresh the history listbox for the specified branch
pub fn refresh_history_list_box(branch_name: String, history_listbox: ListBox) -> io::Result<()> {
    history_listbox.foreach(|row| {
        history_listbox.remove(row);
    });
    let commit_history = get_commit_history(branch_name)?;

    commit_history.iter().for_each(|elemento| {
        set_list_box_row(&history_listbox, elemento);
    });
    Ok(())
}

pub fn branches_listbox_refresh(branches_listbox: ListBox) -> io::Result<()> {
    branches_listbox.foreach(|row| {
        branches_listbox.remove(row);
    });
    branches_listbox_init(branches_listbox)?;
    Ok(())
}

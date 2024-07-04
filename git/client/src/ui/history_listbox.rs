use crate::plumbing::commit::get_parent_commits;
use crate::plumbing::diff::diff_commit::get_patch_of_tree_diffs;
use crate::plumbing::diff::diff_commit::{diff_commit, differences_beetween_files, Patch};
use crate::plumbing::diff::diff_tree;
use crate::plumbing::heads::__get_head_commit;
use crate::plumbing::log::__log;
use crate::ui::history_listbox::diff_tree::diff_tree;
use crate::ui::principal_window::GitApp;
use gtk::glib::Cast;
use gtk::prelude::TextViewExt;
use gtk::prelude::{BinExt, ContainerExt, LabelExt, ListBoxExt, WidgetExt};
use gtk::traits::TextBufferExt;
use gtk::{ListBox, ListBoxRow, TextView};
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::rc::Weak;
use utils::object::object_db::get_object;
use utils::plumbing::commit::get_commit_root;
use utils::plumbing::ls_tree::ls_tree;

// init history listbox with head commit history
pub fn history_listbox_init(history_listbox: ListBox) -> io::Result<()> {
    match get_head_commit_history() {
        Ok(commit_history) => {
            commit_history.iter().for_each(|elemento| {
                set_list_box_row(&history_listbox, elemento);
                println!("Elemento: {}", elemento);
            });
        }
        Err(_) => {
            println!("Todavía no hay commits para mostrar");
        }
    }

    Ok(())
}

pub fn set_list_box_row(history_listbox: &ListBox, line: &str) {
    let list_row = ListBoxRow::new();
    let label = gtk::Label::new(Some(&parse_log(line)));
    label.set_xalign(0.0);
    list_row.add(&label);
    history_listbox.add(&list_row);
    list_row.show_all();
}

// get head commit history using log command
fn get_head_commit_history() -> io::Result<Vec<String>> {
    // En realidad, se debería mostrar por rama, pero por ahora se deja solo la actual.
    let file = File::open(".git/HEAD")?;
    let hash_commit = __get_head_commit(file)?;
    __log(&hash_commit, &mut HashSet::new())
}

pub fn connect_history_listbox(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let file_list_box = strong_ref.file_listbox.clone();
        strong_ref
            .history_listbox
            .connect_row_selected(move |_, row| {
                match history_commits_listbox_handler(row, file_list_box.clone()) {
                    Ok(()) => {}
                    Err(err) => {
                        println!("Error al mostrar historial de commits: {}", err);
                    }
                }
            });
    } else {
        println!("Error al hacer upgrade");
    }
}

pub fn connect_file_list_box(git_app: &Weak<GitApp>) {
    if let Some(strong_ref) = git_app.upgrade() {
        let file_text_view = strong_ref.file_text_view.clone();
        strong_ref.file_listbox.connect_row_selected(move |_, row| {
            if let Ok(()) = file_listbox_handler(row, file_text_view.clone()) {
            } else {
                println!("Error al seleccionar un archivo");
            }
        });
    } else {
        println!("Error al hacer upgrade");
    }
}

/// Show the difference in the file selected with the parent commit
pub fn file_listbox_handler(row: Option<&ListBoxRow>, file_text_view: TextView) -> io::Result<()> {
    if let Some(selected_row) = row {
        if let Some(label) = selected_row
            .child()
            .and_then(|label| label.downcast::<gtk::Label>().ok())
        {
            let label_file_text = label.text().as_str().to_string();
            println!("Archivo seleccionado: {}", label_file_text);
            let hash = label_file_text.split('|').collect::<Vec<&str>>()[1];
            let mut path_name = label_file_text.split('|').collect::<Vec<&str>>()[0];

            path_name = if path_name.split_whitespace().collect::<Vec<&str>>().len() > 1 {
                path_name.split_whitespace().collect::<Vec<&str>>()[1]
            } else {
                path_name
            };
            println!("path name: {}", path_name);
            let patches = get_patches(hash.trim())?;
            // find exact patch related to the name file
            let patch = patches.iter().find(|patch| patch.path == path_name.trim());
            if let Some(patch) = patch {
                let text = differences_beetween_files(patch);
                if let Some(buffer) = file_text_view.buffer() {
                    buffer.set_text(&text);
                }
            } else {
                println!("No se encontró el archivo");
            }
        }
    }
    Ok(())
}

///Based on commit returns vector of patches that represent the difference between the commit and one of its parents
fn get_patches(hash: &str) -> io::Result<Vec<Patch>> {
    let hash = get_hash(hash);
    let data = get_object(&hash)?;
    if let Some(parents_commits) = get_parent_commits(&data.2) {
        if !parents_commits.is_empty() {
            diff_commit(&parents_commits[0], &hash)
        } else {
            Ok(Vec::new())
        }
    } else {
        let (_, _, commit1) = get_object(&hash)?;
        // Get trees.
        let tree = ls_tree(&get_commit_root(&commit1)?)?;
        // Get the diff between the two trees.
        let diff: Vec<_> = diff_tree("", &tree).collect();
        get_patch_of_tree_diffs(&diff, "")
    }
}

pub fn history_commits_listbox_handler(
    row: Option<&ListBoxRow>,
    file_list_box: ListBox,
) -> io::Result<()> {
    if let Some(selected_row) = row {
        if let Some(label) = selected_row
            .child()
            .and_then(|label| label.downcast::<gtk::Label>().ok())
        {
            let label_commit_text = label.text().as_str().to_string();
            let hash = get_hash(&label_commit_text);
            let diferences = get_patches(&hash)?;
            clear_list_box_rows(&file_list_box);
            diferences
                .iter()
                .for_each(|elemento: &crate::plumbing::diff::diff_commit::Patch| {
                    let format = format!("{} | {}", elemento.formatted_path(), hash.trim());
                    set_list_box_row_files(&file_list_box, &format);
                });
        }
    }
    Ok(())
}

/// set list box row for text file, to show differences between commits
pub fn set_list_box_row_files(file_listbox: &ListBox, line: &str) {
    let list_row = ListBoxRow::new();
    let label = gtk::Label::new(Some(line));
    list_row.add(&label);
    file_listbox.add(&list_row);
    list_row.show_all();
}

/// clear list box rows
pub fn clear_list_box_rows(list_box: &ListBox) {
    list_box.foreach(|child| {
        list_box.remove(child);
    });
}

/// get hash from label text
fn get_hash(label: &str) -> String {
    let res: Vec<&str> = label.split('|').collect::<Vec<&str>>();
    let hash = if res.len() > 2 {
        let substring_range = 19..60;
        &label[substring_range]
    } else {
        res[0]
    };
    String::from(hash.trim())
}

pub fn parse_log(log: &str) -> String {
    let mut commit_hash = String::new();
    let mut commit_msg = String::new();
    let mut author = String::new();
    let mut date = String::new();

    let lines = log.lines();

    for line in lines {
        if line.starts_with("commit ") {
            commit_hash = line.trim_start_matches("commit ").to_string();
        } else if line.starts_with("Author: ") {
            if let Some(author_section) = line.trim_start_matches("Author: ").split('<').next() {
                author = author_section.trim().to_string();
            }
        } else if line.starts_with("Date:   ") {
            date = line.trim_start_matches("Date:   ").to_string();
            date.truncate(16);
        } else if line.starts_with('\t') {
            commit_msg = line.trim_start_matches('\t').to_string();
            if commit_msg.len() > 50 {
                commit_msg.truncate(50);
            } else {
                commit_msg.push_str(&" ".repeat(50 - commit_msg.len()));
            }
            break;
        }
    }
    let autor = format!("{:<20}", author);
    let mensaje = format!("{:<30}", commit_msg);

    format!("{} | {} | {} | {}", date, commit_hash, autor, mensaje)
}

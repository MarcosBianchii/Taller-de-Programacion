use crate::plumbing::heads::get_head_name;
use crate::ui::current_branch_text_view::show_current_branch;
use crate::ui::{
    add_button, application_window, branch_button, branch_listbox, cat_file_button,
    changes_listbox, checkout_button, clone_button, commit_button, commit_message_entry,
    fetch_button, history_listbox, init_button, ls_tree_button, merge_button, pull_button,
    push_button, rebase_button, remote_button, remove_button, status_button, tag_annotated_button,
    tag_button,
};
use gtk::{glib, prelude::*};
use std::rc::Rc;

/// Enum to represent the different events that should modify the visual state of the app.
pub enum UiEvent {
    NewBranch(String), // name of the branch
    Init,
    Clone,
    Branch, // show all branches
}

pub struct GitApp {
    application_window: gtk::ApplicationWindow,
    pub(crate) init_button: gtk::MenuItem,
    pub(crate) commit_button: gtk::Button,
    pub(crate) commit_entry: gtk::Entry,
    pub(crate) clone_button: gtk::MenuItem,
    pub(crate) path_entry: gtk::Entry,
    pub(crate) status_button: gtk::MenuItem,
    pub(crate) changes_listbox: gtk::ListBox,
    pub(crate) add_button: gtk::MenuItem,
    pub(crate) remove_button: gtk::MenuItem,
    pub(crate) checkout_button: gtk::MenuItem,
    pub(crate) branch_button: gtk::MenuItem,
    pub(crate) cat_file_button: gtk::MenuItem,
    pub(crate) ls_tree_button: gtk::MenuItem,
    pub(crate) remote_button: gtk::MenuItem,
    pub(crate) fetch_button: gtk::MenuItem,
    pub(crate) push_button: gtk::MenuItem,
    pub(crate) merge_button: gtk::MenuItem,
    pub(crate) pull_button: gtk::MenuItem,
    pub(crate) history_listbox: gtk::ListBox,
    pub(crate) file_text_view: gtk::TextView,
    pub(crate) branches_listbox: gtk::ListBox,
    pub(crate) rebase_button: gtk::MenuItem,
    pub(crate) file_listbox: gtk::ListBox,
    pub(crate) tag_annotated_button: gtk::MenuItem,
    pub(crate) tag_button: gtk::MenuItem,
    pub(crate) terminal_text_view: gtk::TextView,
    pub(crate) current_branch_text_view: gtk::TextView,
}

impl GitApp {
    pub fn new() -> Option<Rc<GitApp>> {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            std::process::exit(1);
        }

        let main_window_ui: &str = include_str!("resources/main_window.ui");
        let builder: gtk::Builder = gtk::Builder::from_string(main_window_ui);

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // el transmiter se lo pasamos a branch
        // ApplicationWindow
        let application_window: gtk::ApplicationWindow = builder.object("main_window")?;

        // Init Button
        let init_button: gtk::MenuItem = builder.object("menu_button_init")?;

        // Commit Button
        let commit_button: gtk::Button = builder.object("commit_button")?;

        // Commit Entry
        let commit_entry: gtk::Entry = builder.object("commit_message_entry")?;

        // Clone Button
        let clone_button: gtk::MenuItem = builder.object("menu_button_clone")?;

        // Path Entry
        let path_entry: gtk::Entry = builder.object("path_entry")?;

        // Status Button
        let status_button: gtk::MenuItem = builder.object("menu_button_status")?;

        // Add ListBox
        let changes_listbox: gtk::ListBox = builder.object("changes_listbox")?;

        // Add Button
        let add_button: gtk::MenuItem = builder.object("menu_button_add")?;

        // Remove Button
        let remove_button: gtk::MenuItem = builder.object("menu_button_remove")?;

        // Checkout Button
        let checkout_button: gtk::MenuItem = builder.object("menu_button_checkout")?;

        // Branch Button
        let branch_button: gtk::MenuItem = builder.object("menu_button_branch")?;

        // Cat File Button
        let cat_file_button: gtk::MenuItem = builder.object("menu_button_cat")?;

        // Ls Tree Button
        let ls_tree_button: gtk::MenuItem = builder.object("menu_button_ls_tree")?;

        // Remote Button
        let remote_button: gtk::MenuItem = builder.object("menu_button_remote")?;

        // Fetch Button
        let fetch_button: gtk::MenuItem = builder.object("menu_button_fetch")?;

        // Push Button
        let push_button: gtk::MenuItem = builder.object("menu_button_push")?;

        // Merge Button
        let merge_button: gtk::MenuItem = builder.object("menu_button_merge")?;

        // Pull Button
        let pull_button: gtk::MenuItem = builder.object("menu_button_pull")?;

        // History ListBox
        let history_listbox: gtk::ListBox = builder.object("history_listbox")?;

        // File TextView
        let file_text_view: gtk::TextView = builder.object("file_text_view")?;

        // Branches ListBox
        let branches_listbox: gtk::ListBox = builder.object("branches_listbox")?;

        // Files ListBox
        let file_listbox: gtk::ListBox = builder.object("file_listbox")?;

        // Rebase Button
        let rebase_button: gtk::MenuItem = builder.object("menu_button_rebase")?;

        // Tag Button
        let tag_button: gtk::MenuItem = builder.object("menu_button_tag")?;

        // Tag Annotated Button
        let tag_annotated_button: gtk::MenuItem = builder.object("menu_tag_annotated_button")?;

        // Terminal TextView
        let terminal_text_view: gtk::TextView = builder.object("terminal_text_view")?;

        // Current Branch TextView
        let current_branch_text_view: gtk::TextView = builder.object("current_branch_text_view")?;

        let principal_window = GitApp {
            application_window,
            init_button,
            commit_button,
            commit_entry,
            clone_button,
            path_entry,
            status_button,
            changes_listbox,
            add_button,
            remove_button,
            checkout_button,
            branch_button,
            cat_file_button,
            ls_tree_button,
            remote_button,
            fetch_button,
            push_button,
            merge_button,
            pull_button,
            history_listbox,
            file_text_view,
            branches_listbox,
            rebase_button,
            file_listbox,
            tag_button,
            tag_annotated_button,
            terminal_text_view,
            current_branch_text_view,
        };

        application_window::initialize_window(&principal_window.application_window);

        let principal_window = Rc::new(principal_window);
        let weak_ref = Rc::downgrade(&principal_window);

        if let Ok(name) = get_head_name() {
            show_current_branch(name, principal_window.current_branch_text_view.clone());
        }

        // Init button
        let tx_clone_init: glib::Sender<UiEvent> = tx.clone();
        init_button::connect_init_button(&weak_ref, tx_clone_init);

        principal_window.commit_button.set_sensitive(false);

        // Commit button handler

        commit_button::connect_commit_button(&weak_ref);

        // Commit message handler
        commit_message_entry::connect_commit_message_entry(&weak_ref);

        // Clone handler
        let tx_clone_for_clone = tx.clone();
        clone_button::connect_clone_button(&weak_ref, tx_clone_for_clone);

        // Status handler
        status_button::connect_status_button(&weak_ref);

        // Add ListBox handler
        changes_listbox::connect_changes_listbox(&weak_ref);

        // Add Button handler
        add_button::connect_add_button(&weak_ref);

        // Remove Button handler
        remove_button::connect_remove_button(&weak_ref);

        // Checkout Button handler
        checkout_button::connect_checkout_button(&weak_ref);

        // Branch Button handler
        let tx_for_branch: glib::Sender<UiEvent> = tx.clone();
        branch_button::connect_branch_button(&weak_ref, tx_for_branch);

        // Cat File Button handler
        cat_file_button::connect_cat_file_button(&weak_ref);

        // Ls Tree Button handler
        ls_tree_button::connect_ls_tree_button(&weak_ref);

        // Remove Button handler
        remote_button::connect_remote_button(&weak_ref);

        // Fetch Button handler
        fetch_button::connect_fetch_button(&weak_ref);

        // Push Button handler
        push_button::connect_push_button(&weak_ref);

        // Merge Button handler
        merge_button::connect_merge_button(&weak_ref);

        // Pull Button handler
        pull_button::connect_pull_button(&weak_ref);

        // Branch listbox handler
        branch_listbox::connect_branch_listbox(&weak_ref);

        // History listbox handler
        history_listbox::connect_history_listbox(&weak_ref);

        // Rebase Button handler
        rebase_button::connect_rebase_button(&weak_ref);

        history_listbox::connect_file_list_box(&weak_ref);

        // Setup application main loop
        let principal_window_clone = principal_window.clone();
        rx.attach(None, move |event| {
            match event {
                UiEvent::NewBranch(branch_name) => {
                    match branch_listbox::add_new_branch(
                        principal_window_clone.branches_listbox.clone(),
                        branch_name,
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Error al añadir nueva rama: {}", err)
                        }
                    }
                }
                UiEvent::Init | UiEvent::Clone => {
                    // Init the history listbox
                    match history_listbox::history_listbox_init(
                        principal_window_clone.history_listbox.clone(),
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Error al inicializar history_listbox: {}", err)
                        }
                    }

                    match branch_listbox::branches_listbox_init(
                        principal_window_clone.branches_listbox.clone(),
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Error al inicializar branches_list_box: {}", err)
                        }
                    }
                }
                UiEvent::Branch => {
                    match branch_listbox::branches_listbox_refresh(
                        principal_window_clone.branches_listbox.clone(),
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Error al actualizar branches: {}", err)
                        }
                    }
                }
            }
            Continue(true)
        });

        // Rebase Button Handler
        rebase_button::connect_rebase_button(&weak_ref);

        // Tag Button Handler
        tag_button::connect_tag_button(&weak_ref);

        // Annotated Tag Button Handler
        tag_annotated_button::connect_tag_annotated_button(&weak_ref);

        Some(principal_window)
    }

    pub fn run(&self) {
        gtk::main();
    }
}

use crate::models::gallery_state::GalleryState;
use crate::services::directory_scanner;
use gtk4::prelude::*;
use gtk4::{
    gio, glib, Align, AlertDialog, Button, EventControllerKey, FileDialog, FileLauncher, FlowBox,
    Picture, ScrolledWindow, Stack, StackTransitionType,
};
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, StatusPage};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

fn load_image_state(
    path: PathBuf,
    state: &Rc<RefCell<GalleryState>>,
    viewer_picture: &Picture,
    flowbox: &FlowBox,
    stack: &Stack,
) {
    let images = if let Some(parent) = path.parent() {
        directory_scanner::get_images_in_dir(parent)
    } else {
        vec![path.clone()]
    };

    *state.borrow_mut() = GalleryState::new(images, Some(&path));

    if let Some(img_path) = state.borrow().current_image() {
        viewer_picture.set_filename(Some(img_path));
        stack.set_visible_child_name("viewer");

        while let Some(child) = flowbox.first_child() {
            flowbox.remove(&child);
        }

        for img in &state.borrow().images {
            let thumb = Picture::builder().width_request(160).height_request(160).can_shrink(true).build();
            thumb.set_filename(Some(img));
            let frame = gtk4::Frame::builder().child(&thumb).css_classes(vec!["view"]).build();
            flowbox.insert(&frame, -1);
        }
    } else {
        stack.set_visible_child_name("empty");
    }
}

pub fn build_ui(app: &Application, initial_file: Option<PathBuf>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Pixora")
        .default_width(1024)
        .default_height(768)
        .build();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    
    let header_bar = HeaderBar::builder().build();
    
    // Header actions
    let open_btn = Button::builder().label("Open").build();
    let copy_btn = Button::builder().icon_name("edit-copy").tooltip_text("Copy Path").build();
    let reveal_btn = Button::builder().icon_name("folder-open").tooltip_text("Reveal in Folder").build();
    let delete_btn = Button::builder().icon_name("user-trash").tooltip_text("Delete Image").css_classes(vec!["destructive-action"]).build();

    header_bar.pack_start(&open_btn);
    header_bar.pack_end(&delete_btn);
    header_bar.pack_end(&reveal_btn);
    header_bar.pack_end(&copy_btn);
    vbox.append(&header_bar);

    let stack = Stack::builder().transition_type(StackTransitionType::Crossfade).hexpand(true).vexpand(true).build();
    vbox.append(&stack);
    window.set_content(Some(&vbox));

    let empty_page = StatusPage::builder().icon_name("image-missing").title("No Image Selected").description("Click Open or press Ctrl+O.").build();
    stack.add_named(&empty_page, Some("empty"));

    let flowbox = FlowBox::builder().valign(Align::Start).max_children_per_line(12).min_children_per_line(2).selection_mode(gtk4::SelectionMode::None).column_spacing(12).row_spacing(12).margin_top(16).margin_bottom(16).margin_start(16).margin_end(16).build();
    let scroll_gallery = ScrolledWindow::builder().child(&flowbox).hscrollbar_policy(gtk4::PolicyType::Never).build();
    stack.add_named(&scroll_gallery, Some("gallery"));

    let viewer_picture = Picture::builder().can_shrink(true).hexpand(true).vexpand(true).build();
    stack.add_named(&viewer_picture, Some("viewer"));

    let state = Rc::new(RefCell::new(GalleryState::new(vec![], None)));
    let slideshow_active = Rc::new(RefCell::new(false));

    if let Some(path) = initial_file {
        load_image_state(path, &state, &viewer_picture, &flowbox, &stack);
    } else {
        stack.set_visible_child_name("empty");
    }

    // --- Button Actions ---

    // 1. Open
    let win_clone = window.clone();
    let state_c = state.clone();
    let viewer_c = viewer_picture.clone();
    let flowbox_c = flowbox.clone();
    let stack_c = stack.clone();
    
    open_btn.connect_clicked(move |_| {
        let dialog = FileDialog::builder().title("Open Image").build();
        let state_cc = state_c.clone();
        let viewer_cc = viewer_c.clone();
        let flowbox_cc = flowbox_c.clone();
        let stack_cc = stack_c.clone();
        
        dialog.open(Some(&win_clone), gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    load_image_state(path, &state_cc, &viewer_cc, &flowbox_cc, &stack_cc);
                }
            }
        });
    });

    // 2. Copy Path
    let state_copy = state.clone();
    let win_copy = window.clone();
    copy_btn.connect_clicked(move |_| {
        if let Some(img_path) = state_copy.borrow().current_image() {
            if let Some(path_str) = img_path.to_str() {
                win_copy.clipboard().set_text(path_str);
            }
        }
    });

    // 3. Reveal in Folder (Open External)
    let state_reveal = state.clone();
    let win_reveal = window.clone();
    reveal_btn.connect_clicked(move |_| {
        if let Some(img_path) = state_reveal.borrow().current_image() {
            let file = gio::File::for_path(img_path);
            let launcher = FileLauncher::new(Some(&file));
            launcher.open_containing_folder(Some(&win_reveal), gio::Cancellable::NONE, |_| {});
        }
    });

    // 4. Safe Delete (Refactored to use gtk4::AlertDialog)
    let state_delete = state.clone();
    let win_delete = window.clone();
    let viewer_delete = viewer_picture.clone();
    let flowbox_delete = flowbox.clone();
    let stack_delete = stack.clone();

    delete_btn.connect_clicked(move |_| {
        let current_path = state_delete.borrow().current_image().cloned();
        
        if let Some(path) = current_path {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            // Use .message() instead of .heading() for gtk4::AlertDialog
            let dialog = AlertDialog::builder()
                .message("Delete Image?")
                .detail(format!("Are you sure you want to delete '{}'? This cannot be undone.", filename))
                .buttons(["Cancel", "Delete"])
                .cancel_button(0)
                .default_button(1)
                .build();
            
            let state_d = state_delete.clone();
            let viewer_d = viewer_delete.clone();
            let flowbox_d = flowbox_delete.clone();
            let stack_d = stack_delete.clone();
            
            dialog.choose(Some(&win_delete), gio::Cancellable::NONE, move |res| {
                // 1 corresponds to the index of "Delete" in the buttons array
                if let Ok(1) = res {
                    if fs::remove_file(&path).is_ok() {
                        // Reload state to reflect deletion
                        let parent = path.parent().unwrap_or(&path).to_path_buf();
                        load_image_state(parent, &state_d, &viewer_d, &flowbox_d, &stack_d);
                    }
                }
            });
        }
    });

    // --- Keyboard Navigation ---
    let key_controller = EventControllerKey::new();
    let pic_clone = viewer_picture.clone();
    let state_clone = state.clone();
    let stack_clone = stack.clone();
    let slideshow_clone = slideshow_active.clone();
    let win_clone_key = window.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        let key_name = key.name().unwrap_or_default();
        
        if key_name == "o" && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
            open_btn.emit_clicked();
            return glib::Propagation::Stop;
        }

        match key_name.as_str() {
            "Left" => {
                if stack_clone.visible_child_name().unwrap_or_default() == "viewer" {
                    if let Some(prev) = state_clone.borrow_mut().prev().cloned() {
                        pic_clone.set_filename(Some(&prev));
                    }
                }
                glib::Propagation::Stop
            }
            "Right" => {
                if stack_clone.visible_child_name().unwrap_or_default() == "viewer" {
                    if let Some(next) = state_clone.borrow_mut().next().cloned() {
                        pic_clone.set_filename(Some(&next));
                    }
                }
                glib::Propagation::Stop
            }
            "g" | "G" => {
                let current = stack_clone.visible_child_name().unwrap_or_default();
                if current == "viewer" {
                    stack_clone.set_visible_child_name("gallery");
                } else if current == "gallery" {
                    stack_clone.set_visible_child_name("viewer");
                }
                glib::Propagation::Stop
            }
            "f" | "F" => {
                if win_clone_key.is_fullscreen() {
                    win_clone_key.unfullscreen();
                } else {
                    win_clone_key.fullscreen();
                }
                glib::Propagation::Stop
            }
            "s" | "S" => {
                let mut active = slideshow_clone.borrow_mut();
                *active = !*active;
                
                if *active {
                    let s_state = state_clone.clone();
                    let s_pic = pic_clone.clone();
                    let s_active = slideshow_clone.clone();
                    
                    glib::timeout_add_seconds_local(3, move || {
                        if !*s_active.borrow() {
                            return glib::ControlFlow::Break;
                        }
                        let mut state_mut = s_state.borrow_mut();
                        let next_path = state_mut.next().cloned().or_else(|| {
                            if !state_mut.images.is_empty() {
                                state_mut.current_index = Some(0);
                                state_mut.current_image().cloned()
                            } else {
                                None
                            }
                        });

                        if let Some(next) = next_path {
                            s_pic.set_filename(Some(&next));
                        }
                        glib::ControlFlow::Continue
                    });
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });

    window.add_controller(key_controller);
    window.present();
}

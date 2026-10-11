use crate::models::gallery_state::GalleryState;
use crate::services::directory_scanner;
use gtk4::prelude::*;
use gtk4::{
    glib, Align, EventControllerKey, FlowBox, Picture, ScrolledWindow, Stack, StackTransitionType,
};
use libadwaita::prelude::*; // Brings in traits like AdwApplicationWindowExt
use libadwaita::{Application, ApplicationWindow, HeaderBar, StatusPage}; // Removed unused ViewStack
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub fn build_ui(app: &Application, initial_file: Option<PathBuf>) {
    // 1. Setup the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Pixora")
        .default_width(1024)
        .default_height(768)
        .build();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // 2. HeaderBar (Top toolbar)
    let header_bar = HeaderBar::builder().build();
    vbox.append(&header_bar);

    // 3. Main content Stack
    let stack = Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .hexpand(true)
        .vexpand(true)
        .build();
    vbox.append(&stack);
    window.set_content(Some(&vbox)); // Now works because of libadwaita::prelude::*

    // --- Create Views ---

    // View A: Empty State
    let empty_page = StatusPage::builder()
        .icon_name("image-missing")
        .title("No Image Selected")
        .description("Pass an image path via CLI to view it.")
        .build();
    stack.add_named(&empty_page, Some("empty"));

    // View B: Gallery Grid
    let flowbox = FlowBox::builder()
        .valign(Align::Start)
        .max_children_per_line(12)
        .min_children_per_line(2)
        .selection_mode(gtk4::SelectionMode::None)
        .column_spacing(12)
        .row_spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let scroll_gallery = ScrolledWindow::builder()
        .child(&flowbox)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build();
    stack.add_named(&scroll_gallery, Some("gallery"));

    // View C: Single Image Viewer
    let viewer_picture = Picture::builder()
        .can_shrink(true)
        .hexpand(true)
        .vexpand(true)
        .build();
    stack.add_named(&viewer_picture, Some("viewer"));

    // --- State Initialization ---
    let state_inner = if let Some(path) = initial_file {
        if let Some(parent) = path.parent() {
            let images = directory_scanner::get_images_in_dir(parent);
            GalleryState::new(images, Some(&path))
        } else {
            GalleryState::new(vec![path.clone()], Some(&path))
        }
    } else {
        GalleryState::new(vec![], None)
    };

    let state = Rc::new(RefCell::new(state_inner));

    // --- Populate UI based on State ---
    let current_state = state.borrow();
    if let Some(img_path) = current_state.current_image() {
        viewer_picture.set_filename(Some(img_path));
        stack.set_visible_child_name("viewer");

        // Populate Gallery Grid
        for img in &current_state.images {
            let thumb = Picture::builder()
                .width_request(160)
                .height_request(160)
                .can_shrink(true) // Replaced content_fit for compatibility
                .build();
            
            thumb.set_filename(Some(img));

            // Wrap in a frame for nice borders
            let frame = gtk4::Frame::builder()
                .child(&thumb)
                .css_classes(vec!["view"])
                .build();

            flowbox.insert(&frame, -1);
        }
    } else {
        stack.set_visible_child_name("empty");
    }
    drop(current_state);

    // --- Keyboard Navigation (Viewer mode) ---
    let key_controller = EventControllerKey::new();
    let pic_clone = viewer_picture.clone();
    let state_clone = state.clone();
    let stack_clone = stack.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        match key.name().unwrap_or_default().as_str() {
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
            _ => glib::Propagation::Proceed,
        }
    });

    window.add_controller(key_controller);
    window.present();
}

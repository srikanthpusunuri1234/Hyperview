use crate::models::gallery_state::GalleryState;
use crate::services::directory_scanner;
use gtk4::prelude::*;
use gtk4::{
    glib, Align, Button, EventControllerKey, FileDialog, FlowBox, Picture, ScrolledWindow, Stack,
    StackTransitionType,
};
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, StatusPage};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// Helper to reload the UI when a new file is opened
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

        // Clear existing thumbnails
        while let Some(child) = flowbox.first_child() {
            flowbox.remove(&child);
        }

        // Populate new thumbnails
        for img in &state.borrow().images {
            let thumb = Picture::builder()
                .width_request(160)
                .height_request(160)
                .can_shrink(true)
                .build();
            thumb.set_filename(Some(img));

            let frame = gtk4::Frame::builder().child(&thumb).css_classes(vec!["view"]).build();
            flowbox.insert(&frame, -1);
        }
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
    
    // HeaderBar with Open Button
    let header_bar = HeaderBar::builder().build();
    let open_btn = Button::builder().label("Open").build();
    header_bar.pack_start(&open_btn);
    vbox.append(&header_bar);

    let stack = Stack::builder().transition_type(StackTransitionType::Crossfade).hexpand(true).vexpand(true).build();
    vbox.append(&stack);
    window.set_content(Some(&vbox));

    let empty_page = StatusPage::builder().icon_name("image-missing").title("No Image Selected").description("Click Open or press Ctrl+O to select an image.").build();
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

    // Open File Dialog Logic
    let win_clone = window.clone();
    let state_clone = state.clone();
    let viewer_clone = viewer_picture.clone();
    let flowbox_clone = flowbox.clone();
    let stack_clone = stack.clone();
    
    open_btn.connect_clicked(move |_| {
        let dialog = FileDialog::builder().title("Open Image").build();
        let state_c = state_clone.clone();
        let viewer_c = viewer_clone.clone();
        let flowbox_c = flowbox_clone.clone();
        let stack_c = stack_clone.clone();
        
        // Fix: Use gtk4::gio::Cancellable instead of glib::gio
        dialog.open(Some(&win_clone), gtk4::gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    load_image_state(path, &state_c, &viewer_c, &flowbox_c, &stack_c);
                }
            }
        });
    });

    // Keyboard Shortcuts (Navigation, Slideshow, Fullscreen)
    let key_controller = EventControllerKey::new();
    let pic_clone = viewer_picture.clone();
    let state_clone = state.clone();
    let stack_clone = stack.clone();
    let slideshow_clone = slideshow_active.clone();
    let win_clone_key = window.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        let key_name = key.name().unwrap_or_default();
        
        // Ctrl+O to open
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
                            return glib::ControlFlow::Break; // Stop slideshow
                        }
                        // Try next image, if none (end of folder), wrap to start
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

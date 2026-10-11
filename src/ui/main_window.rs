use crate::models::gallery_state::GalleryState;
use crate::services::directory_scanner;
use gtk4::prelude::*;
use gtk4::{
    gio, glib, Align, AlertDialog, Button, EventControllerKey, EventControllerScroll,
    EventControllerScrollFlags, FileDialog, FileLauncher, FlowBox, GestureClick, Label, Overlay,
    Picture, ScrolledWindow, Stack, StackTransitionType,
};
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow, HeaderBar, StatusPage};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

fn update_status_overlay(state: &GalleryState, status_label: &Label) {
    if let (Some(index), Some(path)) = (state.current_index, state.current_image()) {
        let total = state.images.len();
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        status_label.set_text(&format!("{} of {}  —  {}", index + 1, total, filename));
    } else {
        status_label.set_text("");
    }
}

fn load_image_state(
    path: PathBuf,
    state: &Rc<RefCell<GalleryState>>,
    viewer_picture: &Picture,
    flowbox: &FlowBox,
    stack: &Stack,
    status_label: &Label,
    current_load_id: &Rc<RefCell<u64>>,
) {
    let images = if let Some(parent) = path.parent() {
        directory_scanner::get_images_in_dir(parent)
    } else {
        vec![path.clone()]
    };

    *state.borrow_mut() = GalleryState::new(images, Some(&path));
    update_status_overlay(&state.borrow(), status_label);

    if let Some(img_path) = state.borrow().current_image() {
        viewer_picture.set_filename(Some(img_path));
        stack.set_visible_child_name("viewer");

        while let Some(child) = flowbox.first_child() {
            flowbox.remove(&child);
        }

        // --- MEMORY-OPTIMIZED WORKER PIPELINE ---
        
        *current_load_id.borrow_mut() += 1;
        let active_id = *current_load_id.borrow();
        
        let current_idx = state.borrow().current_index.unwrap_or(0);
        let total = state.borrow().images.len();
        
        // Memory Optimization: Only load a sliding window of 100 thumbnails maximum
        let start_idx = current_idx.saturating_sub(50);
        let end_idx = (current_idx + 50).min(total);
        let images_to_load = state.borrow().images[start_idx..end_idx].to_vec();
        
        // Use standard Rust MPSC channel to avoid GTK deprecation warnings
        let (tx, rx) = std::sync::mpsc::channel();
        
        thread::spawn(move || {
            for img in images_to_load {
                if tx.send(img).is_err() { break; }
                thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        
        let flowbox_clone = flowbox.clone();
        let load_id_clone = current_load_id.clone();
        
        // Process the queue on the GTK main thread during idle cycles
        glib::idle_add_local(move || {
            if *load_id_clone.borrow() != active_id {
                return glib::ControlFlow::Break;
            }
            
            // Process up to 3 thumbnails per UI frame to keep animations buttery smooth
            for _ in 0..3 {
                match rx.try_recv() {
                    Ok(img_path) => {
                        let thumb = Picture::builder().width_request(160).height_request(160).can_shrink(true).build();
                        thumb.set_filename(Some(&img_path));
                        let frame = gtk4::Frame::builder().child(&thumb).css_classes(vec!["view"]).build();
                        flowbox_clone.insert(&frame, -1);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => return glib::ControlFlow::Continue,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => return glib::ControlFlow::Break,
                }
            }
            glib::ControlFlow::Continue
        });

    } else {
        stack.set_visible_child_name("empty");
    }
}

pub fn build_ui(app: &Application, initial_file: Option<PathBuf>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyperview")
        .default_width(1024)
        .default_height(768)
        .build();

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    
    let header_bar = HeaderBar::builder().build();
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

    let viewer_overlay = Overlay::builder().build();
    let viewer_scrolled = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    let viewer_picture = Picture::builder().can_shrink(true).build();
    
    viewer_scrolled.set_child(Some(&viewer_picture));
    viewer_overlay.set_child(Some(&viewer_scrolled));

    let status_label = Label::builder()
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(24)
        .css_classes(vec!["osd"])
        .build();
    
    viewer_overlay.add_overlay(&status_label);
    stack.add_named(&viewer_overlay, Some("viewer"));

    let state = Rc::new(RefCell::new(GalleryState::new(vec![], None)));
    let slideshow_active = Rc::new(RefCell::new(false));
    let current_zoom = Rc::new(RefCell::new(1.0));
    let current_load_id = Rc::new(RefCell::new(0u64));

    if let Some(path) = initial_file {
        load_image_state(path, &state, &viewer_picture, &flowbox, &stack, &status_label, &current_load_id);
    } else {
        stack.set_visible_child_name("empty");
    }

    // --- Button Actions ---

    let win_clone = window.clone();
    let state_c = state.clone();
    let viewer_c = viewer_picture.clone();
    let flowbox_c = flowbox.clone();
    let stack_c = stack.clone();
    let label_c = status_label.clone();
    let load_c = current_load_id.clone();
    
    open_btn.connect_clicked(move |_| {
        let dialog = FileDialog::builder().title("Open Image").build();
        let state_cc = state_c.clone();
        let viewer_cc = viewer_c.clone();
        let flowbox_cc = flowbox_c.clone();
        let stack_cc = stack_c.clone();
        let label_cc = label_c.clone();
        let load_cc = load_c.clone();
        
        dialog.open(Some(&win_clone), gio::Cancellable::NONE, move |res| {
            if let Ok(file) = res {
                if let Some(path) = file.path() {
                    load_image_state(path, &state_cc, &viewer_cc, &flowbox_cc, &stack_cc, &label_cc, &load_cc);
                }
            }
        });
    });

    let state_copy = state.clone();
    let win_copy = window.clone();
    copy_btn.connect_clicked(move |_| {
        if let Some(img_path) = state_copy.borrow().current_image() {
            if let Some(path_str) = img_path.to_str() {
                win_copy.clipboard().set_text(path_str);
            }
        }
    });

    let state_reveal = state.clone();
    let win_reveal = window.clone();
    reveal_btn.connect_clicked(move |_| {
        if let Some(img_path) = state_reveal.borrow().current_image() {
            let file = gio::File::for_path(img_path);
            let launcher = FileLauncher::new(Some(&file));
            launcher.open_containing_folder(Some(&win_reveal), gio::Cancellable::NONE, |_| {});
        }
    });

    let state_delete = state.clone();
    let win_delete = window.clone();
    let viewer_delete = viewer_picture.clone();
    let flowbox_delete = flowbox.clone();
    let stack_delete = stack.clone();
    let label_delete = status_label.clone();
    let load_delete = current_load_id.clone();

    delete_btn.connect_clicked(move |_| {
        let current_path = state_delete.borrow().current_image().cloned();
        
        if let Some(path) = current_path {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            
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
            let label_d = label_delete.clone();
            let load_d = load_delete.clone();
            
            dialog.choose(Some(&win_delete), gio::Cancellable::NONE, move |res| {
                if let Ok(1) = res {
                    if fs::remove_file(&path).is_ok() {
                        let parent = path.parent().unwrap_or(&path).to_path_buf();
                        load_image_state(parent, &state_d, &viewer_d, &flowbox_d, &stack_d, &label_d, &load_d);
                    }
                }
            });
        }
    });

    // --- Mouse Zoom Controls ---

    let mouse_x = Rc::new(RefCell::new(0.0f64));
    let mouse_y = Rc::new(RefCell::new(0.0f64));
    let motion_controller = gtk4::EventControllerMotion::new();
    
    let mx_clone = mouse_x.clone();
    let my_clone = mouse_y.clone();
    motion_controller.connect_motion(move |_, x, y| {
        *mx_clone.borrow_mut() = x;
        *my_clone.borrow_mut() = y;
    });
    viewer_overlay.add_controller(motion_controller);

    let apply_pointer_zoom = {
        let pic = viewer_picture.clone();
        let scroll_win = viewer_scrolled.clone();
        let mx = mouse_x.clone();
        let my = mouse_y.clone();
        
        move |zooming_in: bool| {
            let hadj = scroll_win.hadjustment();
            let vadj = scroll_win.vadjustment();
            
            let old_width = pic.width() as f64;
            let old_height = pic.height() as f64;
            let pointer_x = *mx.borrow();
            let pointer_y = *my.borrow();
            
            let relative_x = (hadj.value() + pointer_x) / old_width.max(1.0);
            let relative_y = (vadj.value() + pointer_y) / old_height.max(1.0);

            if zooming_in {
                pic.set_can_shrink(false); 
            } else {
                pic.set_can_shrink(true);  
            }

            let pic_clone = pic.clone();
            glib::idle_add_local(move || {
                let new_width = pic_clone.width() as f64;
                let new_height = pic_clone.height() as f64;
                
                if (new_width - old_width).abs() > 1.0 || (new_height - old_height).abs() > 1.0 {
                    let target_x = (new_width * relative_x) - pointer_x;
                    let target_y = (new_height * relative_y) - pointer_y;
                    
                    hadj.set_value(target_x.clamp(hadj.lower(), hadj.upper() - hadj.page_size()));
                    vadj.set_value(target_y.clamp(vadj.lower(), vadj.upper() - vadj.page_size()));
                }
                glib::ControlFlow::Break
            });
        }
    };

    let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
    let zoom_apply_scroll = apply_pointer_zoom.clone();
    
    scroll_controller.connect_scroll(move |ctrl, _dx, dy| {
        let state = ctrl.current_event_state();
        if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
            if dy < 0.0 {
                zoom_apply_scroll(true); 
            } else if dy > 0.0 {
                zoom_apply_scroll(false); 
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });

    let click_gesture = GestureClick::new();
    click_gesture.set_button(0);
    let zoom_apply_click = apply_pointer_zoom.clone();
    let pic_click = viewer_picture.clone();
    
    click_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
        if n_press == 2 {
            let currently_zoomed = !pic_click.can_shrink();
            zoom_apply_click(!currently_zoomed);
        }
    });

    viewer_overlay.add_controller(scroll_controller);
    viewer_overlay.add_controller(click_gesture);

    // --- Keyboard Navigation ---

    let key_controller = EventControllerKey::new();
    let pic_clone = viewer_picture.clone();
    let state_clone = state.clone();
    let stack_clone = stack.clone();
    let label_clone = status_label.clone();
    let slideshow_clone = slideshow_active.clone();
    let win_clone_key = window.clone();
    let zoom_clone = current_zoom.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        let key_name = key.name().unwrap_or_default();
        
        if key_name == "o" && modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
            open_btn.emit_clicked();
            return glib::Propagation::Stop;
        }

        match key_name.as_str() {
            "Left" => {
                if stack_clone.visible_child_name().unwrap_or_default() == "viewer" {
                    let prev_path = state_clone.borrow_mut().prev().cloned();
                    if let Some(prev) = prev_path {
                        pic_clone.set_filename(Some(&prev));
                        update_status_overlay(&state_clone.borrow(), &label_clone);
                    }
                }
                glib::Propagation::Stop
            }
            "Right" => {
                if stack_clone.visible_child_name().unwrap_or_default() == "viewer" {
                    let next_path = state_clone.borrow_mut().next().cloned();
                    if let Some(next) = next_path {
                        pic_clone.set_filename(Some(&next));
                        update_status_overlay(&state_clone.borrow(), &label_clone);
                    }
                }
                glib::Propagation::Stop
            }
            "0" => {
                *zoom_clone.borrow_mut() = 1.0;
                pic_clone.set_can_shrink(true);
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
                    let s_label = label_clone.clone();
                    
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
                            update_status_overlay(&state_mut, &s_label);
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

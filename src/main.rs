mod models {
    pub mod gallery_state;
}
mod services {
    pub mod directory_scanner;
}
mod ui {
    pub mod main_window;
}

use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use libadwaita::Application;

fn main() {
    // Initialize Libadwaita application with HANDLES_OPEN flag
    let app = Application::builder()
        .application_id("org.pixora.Gallery")
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();

    // Triggered when launched without arguments
    app.connect_activate(|app| {
        ui::main_window::build_ui(app, None);
    });

    // Triggered when launched with a file argument
    app.connect_open(|app, files, _| {
        let initial_file = files.first().and_then(|f| f.path());
        ui::main_window::build_ui(app, initial_file);
    });

    app.run();
}

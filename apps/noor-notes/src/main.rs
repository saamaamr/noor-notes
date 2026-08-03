use adw::prelude::*;

fn main() {
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes")
        .build();

    app.connect_activate(|app| {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Noor Notes")
            .default_width(420)
            .default_height(360)
            .build();
        window.present();
    });

    app.run();
}

pub const fn display_name() -> &'static str {
    if cfg!(feature = "development") {
        "Noor Notes Dev"
    } else {
        "Noor Notes"
    }
}

pub const fn executable_name() -> &'static str {
    if cfg!(feature = "development") {
        "noor-notes-dev"
    } else {
        "noor-notes"
    }
}

pub const fn application_id() -> &'static str {
    if cfg!(feature = "development") {
        "io.github.saamaamr.NoorNotes.Devel"
    } else {
        "io.github.saamaamr.NoorNotes"
    }
}

pub const fn subtitle() -> &'static str {
    if cfg!(feature = "development") {
        "Development build · Private notebook"
    } else {
        "Private notebook"
    }
}

pub fn window_title() -> adw::WindowTitle {
    adw::WindowTitle::new(display_name(), subtitle())
}

pub fn application() -> adw::Application {
    adw::Application::builder()
        .application_id(application_id())
        .build()
}

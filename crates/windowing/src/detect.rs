#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Environment {
    session_type: String,
    current_desktop: String,
}

impl Environment {
    pub fn new(session_type: impl Into<String>, current_desktop: impl Into<String>) -> Self {
        Self {
            session_type: session_type.into(),
            current_desktop: current_desktop.into(),
        }
    }

    pub fn current() -> Self {
        Self::new(
            std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
            std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendKind {
    X11,
    GnomeWayland,
    Fallback,
}

pub fn detect_backend(environment: &Environment) -> BackendKind {
    if environment.session_type.eq_ignore_ascii_case("x11") {
        BackendKind::X11
    } else if environment.session_type.eq_ignore_ascii_case("wayland")
        && environment
            .current_desktop
            .to_ascii_lowercase()
            .contains("gnome")
    {
        BackendKind::GnomeWayland
    } else {
        BackendKind::Fallback
    }
}

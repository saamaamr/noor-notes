use noor_windowing::{BackendKind, Environment, detect_backend};

#[test]
fn x11_session_selects_x11_backend() {
    let environment = Environment::new("x11", "ubuntu:GNOME");

    assert_eq!(detect_backend(&environment), BackendKind::X11);
}

#[test]
fn gnome_wayland_selects_gnome_adapter() {
    let environment = Environment::new("wayland", "ubuntu:GNOME");

    assert_eq!(detect_backend(&environment), BackendKind::GnomeWayland);
}

#[test]
fn unknown_wayland_desktop_uses_safe_fallback() {
    let environment = Environment::new("wayland", "sway");

    assert_eq!(detect_backend(&environment), BackendKind::Fallback);
}

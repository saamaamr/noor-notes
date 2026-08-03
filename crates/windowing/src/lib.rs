//! Desktop-specific sticky-window controls.

mod controller;
mod detect;
mod fallback;
mod x11;

pub use controller::{NativeWindowId, WindowCapabilities, WindowController, WindowError};
pub use detect::{BackendKind, Environment, detect_backend};
pub use fallback::FallbackWindowController;
pub use x11::X11WindowController;

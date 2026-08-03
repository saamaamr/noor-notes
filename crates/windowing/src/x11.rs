use std::sync::Arc;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, PropMode};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

use crate::{NativeWindowId, WindowCapabilities, WindowController, WindowError};

#[derive(Clone)]
pub struct X11WindowController {
    connection: Arc<RustConnection>,
}

impl X11WindowController {
    pub fn connect() -> Result<Self, WindowError> {
        let (connection, _) = x11rb::connect(None).map_err(backend_error)?;
        Ok(Self {
            connection: Arc::new(connection),
        })
    }

    fn atom(&self, name: &[u8]) -> Result<Atom, WindowError> {
        self.connection
            .intern_atom(false, name)
            .map_err(backend_error)?
            .reply()
            .map(|reply| reply.atom)
            .map_err(backend_error)
    }

    fn x11_window(window: NativeWindowId) -> Result<u32, WindowError> {
        match window {
            NativeWindowId::X11(window) => Ok(window),
            NativeWindowId::Wayland(_) => Err(WindowError::UnsupportedOperation),
        }
    }
}

#[async_trait::async_trait]
impl WindowController for X11WindowController {
    async fn set_above(&self, window: NativeWindowId, enabled: bool) -> Result<(), WindowError> {
        let window = Self::x11_window(window)?;
        let state_atom = self.atom(b"_NET_WM_STATE")?;
        let above_atom = self.atom(b"_NET_WM_STATE_ABOVE")?;
        let reply = self
            .connection
            .get_property(false, window, state_atom, AtomEnum::ATOM, 0, u32::MAX)
            .map_err(backend_error)?
            .reply()
            .map_err(backend_error)?;
        let mut states: Vec<u32> = reply.value32().into_iter().flatten().collect();
        states.retain(|state| *state != above_atom);
        if enabled {
            states.push(above_atom);
        }
        self.connection
            .change_property32(
                PropMode::REPLACE,
                window,
                state_atom,
                AtomEnum::ATOM,
                &states,
            )
            .map_err(backend_error)?;
        self.connection.flush().map_err(backend_error)
    }

    async fn set_all_workspaces(
        &self,
        window: NativeWindowId,
        enabled: bool,
    ) -> Result<(), WindowError> {
        let window = Self::x11_window(window)?;
        let desktop_atom = self.atom(b"_NET_WM_DESKTOP")?;
        if enabled {
            self.connection
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    desktop_atom,
                    AtomEnum::CARDINAL,
                    &[u32::MAX],
                )
                .map_err(backend_error)?;
        } else {
            self.connection
                .delete_property(window, desktop_atom)
                .map_err(backend_error)?;
        }
        self.connection.flush().map_err(backend_error)
    }

    async fn set_opacity(&self, window: NativeWindowId, value: f64) -> Result<(), WindowError> {
        let window = Self::x11_window(window)?;
        let opacity_atom = self.atom(b"_NET_WM_WINDOW_OPACITY")?;
        let opacity = (value.clamp(0.35, 1.0) * f64::from(u32::MAX)).round() as u32;
        self.connection
            .change_property32(
                PropMode::REPLACE,
                window,
                opacity_atom,
                AtomEnum::CARDINAL,
                &[opacity],
            )
            .map_err(backend_error)?;
        self.connection.flush().map_err(backend_error)
    }

    fn capabilities(&self) -> WindowCapabilities {
        WindowCapabilities {
            always_on_top: true,
            all_workspaces: true,
            opacity: true,
        }
    }
}

fn backend_error(error: impl std::fmt::Display) -> WindowError {
    WindowError::Backend(error.to_string())
}

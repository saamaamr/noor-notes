use crate::{NativeWindowId, WindowCapabilities, WindowController, WindowError};

const DESTINATION: &str = "io.github.saamaamr.NoorNotes.Windowing";
const OBJECT_PATH: &str = "/io/github/saamaamr/NoorNotes/Window1";
const INTERFACE: &str = "io.github.saamaamr.NoorNotes.Window1";

#[derive(Clone)]
pub struct GnomeWindowController {
    connection: zbus::Connection,
}

impl GnomeWindowController {
    pub async fn connect() -> Result<Self, WindowError> {
        let connection = zbus::Connection::session().await.map_err(backend_error)?;
        let proxy = zbus::fdo::DBusProxy::new(&connection)
            .await
            .map_err(backend_error)?;
        let name = DESTINATION.try_into().map_err(backend_error)?;
        if !proxy.name_has_owner(name).await.map_err(backend_error)? {
            return Err(WindowError::UnsupportedOperation);
        }
        Ok(Self { connection })
    }

    pub const fn service_identity() -> (&'static str, &'static str, &'static str) {
        (DESTINATION, OBJECT_PATH, INTERFACE)
    }

    pub fn window_title(note_id: &str) -> String {
        format!("Noor Note::{note_id}")
    }

    async fn call(
        &self,
        method: &str,
        window: NativeWindowId,
        enabled: bool,
    ) -> Result<(), WindowError> {
        let NativeWindowId::Wayland(window_id) = window else {
            return Err(WindowError::UnsupportedOperation);
        };
        self.connection
            .call_method(
                Some(DESTINATION),
                OBJECT_PATH,
                Some(INTERFACE),
                method,
                &(window_id, enabled),
            )
            .await
            .map_err(backend_error)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WindowController for GnomeWindowController {
    async fn set_above(&self, window: NativeWindowId, enabled: bool) -> Result<(), WindowError> {
        self.call("SetAbove", window, enabled).await
    }

    async fn set_all_workspaces(
        &self,
        window: NativeWindowId,
        enabled: bool,
    ) -> Result<(), WindowError> {
        self.call("SetAllWorkspaces", window, enabled).await
    }

    async fn set_opacity(&self, _window: NativeWindowId, _value: f64) -> Result<(), WindowError> {
        Err(WindowError::UnsupportedOperation)
    }

    fn capabilities(&self) -> WindowCapabilities {
        WindowCapabilities {
            always_on_top: true,
            all_workspaces: true,
            opacity: false,
        }
    }
}

fn backend_error(error: impl std::fmt::Display) -> WindowError {
    WindowError::Backend(error.to_string())
}

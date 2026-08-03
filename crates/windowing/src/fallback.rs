use crate::{NativeWindowId, WindowCapabilities, WindowController, WindowError};

#[derive(Clone, Copy, Debug, Default)]
pub struct FallbackWindowController;

#[async_trait::async_trait]
impl WindowController for FallbackWindowController {
    async fn set_above(&self, _window: NativeWindowId, _enabled: bool) -> Result<(), WindowError> {
        Err(WindowError::UnsupportedOperation)
    }

    async fn set_all_workspaces(
        &self,
        _window: NativeWindowId,
        _enabled: bool,
    ) -> Result<(), WindowError> {
        Err(WindowError::UnsupportedOperation)
    }

    async fn set_opacity(&self, _window: NativeWindowId, _value: f64) -> Result<(), WindowError> {
        Err(WindowError::UnsupportedOperation)
    }

    fn capabilities(&self) -> WindowCapabilities {
        WindowCapabilities::default()
    }
}

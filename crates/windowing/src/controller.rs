#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeWindowId {
    X11(u32),
    Wayland(u64),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowCapabilities {
    pub always_on_top: bool,
    pub all_workspaces: bool,
    pub opacity: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("window operation is unsupported on this desktop")]
    UnsupportedOperation,
    #[error("window backend failed: {0}")]
    Backend(String),
}

#[async_trait::async_trait]
pub trait WindowController {
    async fn set_above(&self, window: NativeWindowId, enabled: bool) -> Result<(), WindowError>;
    async fn set_all_workspaces(
        &self,
        window: NativeWindowId,
        enabled: bool,
    ) -> Result<(), WindowError>;
    async fn set_opacity(&self, window: NativeWindowId, value: f64) -> Result<(), WindowError>;
    fn capabilities(&self) -> WindowCapabilities;
}

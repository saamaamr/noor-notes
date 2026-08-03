use noor_windowing::{FallbackWindowController, NativeWindowId, WindowController};

#[tokio::test]
async fn fallback_reports_unsupported_capabilities_and_operations() {
    let controller = FallbackWindowController;
    let capabilities = controller.capabilities();
    assert!(!capabilities.always_on_top);
    assert!(!capabilities.all_workspaces);
    assert!(!capabilities.opacity);

    let window = NativeWindowId::X11(42);
    assert!(controller.set_above(window, true).await.is_err());
    assert!(controller.set_all_workspaces(window, true).await.is_err());
    assert!(controller.set_opacity(window, 0.8).await.is_err());
}

use noor_windowing::{GnomeWindowController, WindowController};

#[test]
fn gnome_adapter_contract_is_narrow_and_capability_complete() {
    assert_eq!(
        GnomeWindowController::service_identity(),
        (
            "io.github.saamaamr.NoorNotes.Windowing",
            "/io/github/saamaamr/NoorNotes/Window1",
            "io.github.saamaamr.NoorNotes.Window1"
        )
    );
    assert_eq!(
        GnomeWindowController::window_title("018f2f91-8d87-7c4a-a9ee-9b90518f4123"),
        "Noor Note::018f2f91-8d87-7c4a-a9ee-9b90518f4123"
    );
}

#[tokio::test]
async fn absent_extension_fails_probe_without_affecting_fallback() {
    if let Ok(controller) = GnomeWindowController::connect().await {
        let capabilities = controller.capabilities();
        assert!(capabilities.always_on_top);
        assert!(capabilities.all_workspaces);
        assert!(!capabilities.opacity);
    }
}

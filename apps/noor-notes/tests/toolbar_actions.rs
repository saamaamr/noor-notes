const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const MANAGED_APP: &str = include_str!("../src/managed_app.rs");
const TRASH_COMMAND: &str = include_str!("../src/services/trash_command.rs");
const EDITOR_ACTIONS: &str = include_str!("../src/editor_actions.rs");
const DIALOG_PRIMITIVES: &str = include_str!("../src/ui/dialog_primitives.rs");

#[test]
fn standalone_editor_uses_shared_document_chrome_and_commands() {
    assert!(NOTE_WINDOW.contains("EditorHeader::new"));
    assert!(NOTE_WINDOW.contains("EditorMenuBar::new"));
    assert!(NOTE_WINDOW.contains("EditorToolbar::new"));
    assert!(NOTE_WINDOW.contains("crate::editor_actions::connect"));
    assert!(!NOTE_WINDOW.contains("FormattingPopover::new()"));
    for command in [
        "EditorCommand::Undo",
        "EditorCommand::Redo",
        "EditorCommand::Bold",
        "EditorCommand::Italic",
        "EditorCommand::Underline",
    ] {
        assert!(EDITOR_ACTIONS.contains(command), "missing shared {command}");
    }
}

#[test]
fn every_primary_toolbar_button_is_wired() {
    assert!(
        NOTE_WINDOW.contains(".new_note")
            && NOTE_WINDOW.contains("app.activate_action(\"new-note\""),
        "New Note must activate app.new-note"
    );
    assert!(
        NOTE_WINDOW.contains("for button in [&toolbar.header_archive, &toolbar.archive]")
            && NOTE_WINDOW.contains("connect_archive_button("),
        "header and More-menu Archive controls must share one persistence path"
    );
    assert!(
        NOTE_WINDOW.contains("connect_trash_button")
            && NOTE_WINDOW.contains("[&toolbar.header_trash, &toolbar.trash]"),
        "Delete must open its confirmation flow"
    );
    assert!(
        TRASH_COMMAND.contains("Move to Trash") && TRASH_COMMAND.contains("confirm_move_to_trash"),
        "Delete must require explicit confirmation"
    );
    assert!(
        NOTE_WINDOW.contains("autosave.flush"),
        "state-changing actions must save before closing"
    );
    assert!(
        NOTE_WINDOW.contains("refresh-notes") && MANAGED_APP.contains("refresh-notes"),
        "saved lifecycle changes must refresh the main note lists"
    );
    assert!(
        NOTE_WINDOW.contains("mode_switch_busy")
            && NOTE_WINDOW.contains("set_mode_buttons_sensitive"),
        "mode conversion must block duplicate dialogs while an action is running"
    );
}

#[test]
fn more_actions_close_the_popover_before_modal_work() {
    let source = include_str!("../src/ui/editor_toolbar.rs");
    assert!(
        source.contains("close_more_on_click") && source.contains("more_popover.popdown"),
        "More actions must release the popover grab before opening dialogs or windows"
    );
}

#[test]
fn confirmations_and_entry_dialogs_share_one_explicit_contract() {
    for helper in [
        "confirm_action",
        "confirm_destructive",
        "request_text",
        "show_error",
        "popdown_before_dialog",
    ] {
        assert!(
            DIALOG_PRIMITIVES.contains(helper),
            "missing shared dialog helper: {helper}"
        );
    }
    assert!(TRASH_COMMAND.contains("dialog_primitives::confirm_destructive"));
    assert!(NOTE_WINDOW.contains("dialog_primitives::request_text"));
    assert!(NOTE_WINDOW.contains("dialog_primitives::popdown_before_dialog"));
}

#[test]
fn every_modal_more_action_releases_the_parent_popover() {
    use gtk::prelude::*;
    use noor_notes::ui::editor_toolbar::EditorToolbar;

    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    let window = gtk::Window::builder().child(&toolbar.widget).build();
    window.present();
    settle();
    for action in [
        &toolbar.new_note,
        &toolbar.rename,
        &toolbar.duplicate,
        &toolbar.archive,
        &toolbar.trash,
        &toolbar.restore,
        &toolbar.permanent_delete,
        &toolbar.mode_rich,
        &toolbar.mode_markdown,
        &toolbar.mode_plain,
        &toolbar.mode_code,
    ] {
        toolbar.more.popup();
        settle();
        assert!(toolbar.more_popover.is_visible());
        action.emit_clicked();
        assert!(
            !toolbar.more_popover.is_visible(),
            "{} left More open",
            action
                .tooltip_text()
                .unwrap_or_else(|| action.label().unwrap_or_default())
        );
    }
    window.close();
}

fn settle() {
    let context = gtk::glib::MainContext::default();
    // Popover map/unmap and frame-clock events can arrive after pending() is
    // empty, especially while the linker is busy on CI.
    for _ in 0..20 {
        while context.pending() {
            context.iteration(false);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

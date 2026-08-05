#![allow(deprecated)]

pub fn shortcuts_window() -> gtk::ShortcutsWindow {
    let window = gtk::ShortcutsWindow::builder()
        .title("Keyboard Shortcuts")
        .modal(true)
        .build();
    let section = gtk::ShortcutsSection::builder().title("General").build();
    let group = gtk::ShortcutsGroup::builder().title("Noor Notes").build();
    for (title, accelerator) in [
        ("New note", "<Primary>n"),
        ("Search notes", "<Primary>f"),
        ("Quit", "<Primary>q"),
        ("Undo", "<Primary>z"),
        ("Redo", "<Primary><Shift>z"),
        ("Find in note", "<Primary>f"),
        ("Bold", "<Primary>b"),
        ("Italic", "<Primary>i"),
        ("Underline", "<Primary>u"),
    ] {
        group.add_shortcut(
            &gtk::ShortcutsShortcut::builder()
                .title(title)
                .accelerator(accelerator)
                .build(),
        );
    }
    section.add_group(&group);
    window.add_section(&section);
    window
}

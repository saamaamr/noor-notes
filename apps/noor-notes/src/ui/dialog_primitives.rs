use adw::prelude::*;

pub async fn confirm_action(
    parent: &impl IsA<gtk::Widget>,
    heading: &str,
    body: &str,
    accept_label: &str,
) -> bool {
    choose_confirmation(
        parent,
        heading,
        body,
        accept_label,
        adw::ResponseAppearance::Suggested,
    )
    .await
}

pub async fn confirm_destructive(
    parent: &impl IsA<gtk::Widget>,
    heading: &str,
    body: &str,
    accept_label: &str,
) -> bool {
    choose_confirmation(
        parent,
        heading,
        body,
        accept_label,
        adw::ResponseAppearance::Destructive,
    )
    .await
}

pub async fn request_text(
    parent: &impl IsA<gtk::Widget>,
    heading: &str,
    body: &str,
    initial: &str,
    placeholder: &str,
    accept_label: &str,
) -> Option<String> {
    let previous_focus = focused_widget(parent);
    let entry = gtk::Entry::builder()
        .text(initial)
        .placeholder_text(placeholder)
        .activates_default(true)
        .hexpand(true)
        .build();
    entry.update_property(&[gtk::accessible::Property::Label(heading)]);
    let dialog = adw::AlertDialog::builder()
        .heading(heading)
        .body(body)
        .extra_child(&entry)
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("accept", accept_label);
    dialog.set_default_response(Some("accept"));
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("accept", adw::ResponseAppearance::Suggested);
    let accepted = dialog.choose_future(Some(parent)).await == "accept";
    restore_focus(previous_focus);
    accepted.then(|| entry.text().trim().to_owned())
}

pub fn show_error(parent: &impl IsA<gtk::Widget>, heading: &str, body: &str) {
    let dialog = adw::AlertDialog::new(Some(heading), Some(body));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.set_close_response("ok");
    dialog.present(Some(parent));
}

pub fn popdown_before_dialog(popover: &gtk::Popover) {
    if popover.is_visible() {
        popover.popdown();
    }
}

async fn choose_confirmation(
    parent: &impl IsA<gtk::Widget>,
    heading: &str,
    body: &str,
    accept_label: &str,
    appearance: adw::ResponseAppearance,
) -> bool {
    let previous_focus = focused_widget(parent);
    let dialog = adw::AlertDialog::new(Some(heading), Some(body));
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("accept", accept_label);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("accept", appearance);
    let accepted = dialog.choose_future(Some(parent)).await == "accept";
    restore_focus(previous_focus);
    accepted
}

fn focused_widget(parent: &impl IsA<gtk::Widget>) -> Option<gtk::Widget> {
    parent
        .as_ref()
        .root()
        .and_downcast::<gtk::Window>()
        .and_then(|window| gtk::prelude::GtkWindowExt::focus(&window))
}

fn restore_focus(widget: Option<gtk::Widget>) {
    if let Some(widget) = widget.filter(|widget| widget.is_visible()) {
        widget.grab_focus();
    }
}

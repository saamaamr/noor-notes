use adw::prelude::*;

pub fn add_action<F>(app: &adw::Application, name: &str, activate: F)
where
    F: Fn(&gtk::gio::SimpleAction, Option<&gtk::glib::Variant>) + 'static,
{
    let action = gtk::gio::SimpleAction::new(name, None);
    action.connect_activate(activate);
    app.add_action(&action);
}

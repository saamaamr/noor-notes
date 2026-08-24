use adw::prelude::*;
use noor_sync::SyncStatus;

#[derive(Clone)]
pub struct SyncStatusView {
    pub widget: gtk::Box,
    icon: gtk::Image,
    label: gtk::Label,
}

impl SyncStatusView {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let icon = gtk::Image::from_icon_name("view-refresh-symbolic");
        let label = gtk::Label::new(Some("Local only"));
        label.set_xalign(0.0);
        widget.append(&icon);
        widget.append(&label);
        Self {
            widget,
            icon,
            label,
        }
    }

    pub fn update(&self, status: SyncStatus, pending: usize, last_success: Option<&str>) {
        let (icon, message) = match status {
            SyncStatus::Idle if pending == 0 => (
                "object-select-symbolic",
                last_success
                    .map(|value| format!("Synced · {value}"))
                    .unwrap_or_else(|| "Ready to sync".into()),
            ),
            SyncStatus::Idle => (
                "view-refresh-symbolic",
                format!("{pending} changes waiting"),
            ),
            SyncStatus::Syncing => ("view-refresh-symbolic", "Syncing…".into()),
            SyncStatus::Offline => (
                "network-offline-symbolic",
                "Offline · changes are safe".into(),
            ),
            SyncStatus::AuthRequired => (
                "dialog-password-symbolic",
                "Sign in again to resume sync".into(),
            ),
            SyncStatus::Error => (
                "dialog-warning-symbolic",
                "Sync needs attention · local editing is available".into(),
            ),
        };
        self.icon.set_icon_name(Some(icon));
        self.label.set_text(&message);
    }
}

impl Default for SyncStatusView {
    fn default() -> Self {
        Self::new()
    }
}

use std::rc::Rc;

use adw::prelude::*;
use noor_domain::{Note, NoteId, NoteState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardAction {
    Archive,
    Trash,
    Restore,
    DeletePermanently,
}

pub struct NoteCard {
    pub widget: gtk::Box,
    pub menu: gtk::MenuButton,
    action_buttons: Vec<(CardAction, gtk::Button)>,
}

impl NoteCard {
    pub fn action_button(&self, action: CardAction) -> Option<gtk::Button> {
        self.action_buttons
            .iter()
            .find_map(|(candidate, button)| (*candidate == action).then(|| button.clone()))
    }
}

pub fn build(note: &Note, action: Rc<dyn Fn(NoteId, CardAction)>) -> NoteCard {
    let card = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    card.add_css_class("nn-note-card");
    card.add_css_class("nn-p-12");
    card.add_css_class("nn-m-4");
    card.add_css_class("nn-radius-8");
    card.add_css_class("nn-surface");
    card.add_css_class(note.color.css_class());
    let color = gtk::Box::new(gtk::Orientation::Vertical, 0);
    color.add_css_class("nn-color-strip");
    color.set_width_request(4);
    card.append(&color);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 6);
    text.add_css_class("nn-note-card-content");
    text.set_hexpand(true);
    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let title = gtk::Label::new(Some(note.display_title()));
    title.add_css_class("nn-note-title");
    title.set_xalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.set_hexpand(true);
    heading.append(&title);
    if note.pinned {
        let icon = gtk::Image::from_icon_name("view-pin-symbolic");
        icon.add_css_class("nn-note-status-icon");
        icon.add_css_class("nn-icon-secondary");
        icon.set_tooltip_text(Some("Pinned note"));
        icon.update_property(&[gtk::accessible::Property::Label("Pinned note")]);
        heading.append(&icon);
    }
    if note.favorite {
        let icon = gtk::Image::from_icon_name("starred-symbolic");
        icon.add_css_class("nn-note-status-icon");
        icon.add_css_class("nn-icon-secondary");
        icon.set_tooltip_text(Some("Favorite note"));
        icon.update_property(&[gtk::accessible::Property::Label("Favorite note")]);
        heading.append(&icon);
    }
    text.append(&heading);
    let preview = gtk::Label::new(Some(&crate::library_view::content_preview(
        &note.content,
        140,
    )));
    preview.add_css_class("nn-metadata");
    preview.add_css_class("nn-note-card-preview");
    preview.add_css_class("nn-text-meta");
    preview.set_xalign(0.0);
    preview.set_lines(2);
    preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
    preview.set_wrap(true);
    preview.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    text.append(&preview);
    let tags = gtk::Label::new(Some(
        &note
            .tags
            .iter()
            .take(2)
            .map(|tag| format!("#{tag}"))
            .collect::<Vec<_>>()
            .join("   "),
    ));
    tags.add_css_class("nn-note-card-tags");
    tags.set_xalign(0.0);
    tags.set_ellipsize(gtk::pango::EllipsizeMode::End);
    tags.set_visible(!note.tags.is_empty());
    text.append(&tags);
    let meta = gtk::Label::new(Some(
        &note
            .updated_at
            .format("Edited %d %b · %I:%M %p")
            .to_string(),
    ));
    meta.add_css_class("nn-caption");
    meta.add_css_class("nn-note-card-meta");
    meta.set_xalign(0.0);
    meta.set_ellipsize(gtk::pango::EllipsizeMode::End);
    text.append(&meta);
    card.append(&text);
    let actions = match note.state {
        NoteState::Active => vec![
            ("Archive", CardAction::Archive, false),
            ("Move to Trash", CardAction::Trash, true),
        ],
        NoteState::Archived => vec![
            ("Restore to All Notes", CardAction::Restore, false),
            ("Move to Trash", CardAction::Trash, true),
        ],
        NoteState::Trashed { .. } => vec![
            ("Restore", CardAction::Restore, false),
            ("Delete permanently", CardAction::DeletePermanently, true),
        ],
    };
    let popover_content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    popover_content.set_margin_top(6);
    popover_content.set_margin_bottom(6);
    popover_content.set_margin_start(6);
    popover_content.set_margin_end(6);
    let popover = gtk::Popover::new();
    super::popover_primitives::style_popover(&popover);
    let mut action_buttons = Vec::new();
    let mut separated_destructive_actions = false;
    for (label, card_action, destructive) in actions {
        if destructive && !separated_destructive_actions {
            let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            separator.add_css_class("nn-menu-separator");
            popover_content.append(&separator);
            separated_destructive_actions = true;
        }
        let button = gtk::Button::with_label(label);
        button.set_halign(gtk::Align::Fill);
        button.set_tooltip_text(Some(label));
        button.update_property(&[gtk::accessible::Property::Label(label)]);
        button.add_css_class("flat");
        button.add_css_class("nn-menu-row");
        if destructive {
            button.add_css_class("destructive-action");
            button.add_css_class("nn-menu-danger");
        }
        let action = action.clone();
        let id = note.id;
        let action_popover = popover.clone();
        button.connect_clicked(move |_| {
            action_popover.popdown();
            action(id, card_action);
        });
        action_buttons.push((card_action, button.clone()));
        popover_content.append(&button);
    }
    popover.set_child(Some(&popover_content));
    let menu = gtk::MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text("Note actions")
        .popover(&popover)
        .build();
    menu.add_css_class("flat");
    menu.add_css_class("nn-card-action");
    menu.add_css_class("nn-icon-button");
    menu.add_css_class("nn-h-32");
    menu.add_css_class("nn-radius-8");
    menu.add_css_class("nn-focus-ring");
    menu.set_valign(gtk::Align::Center);
    menu.update_property(&[gtk::accessible::Property::Label("Note actions")]);
    let gesture = gtk::GestureClick::new();
    gesture.set_button(3);
    let popover = popover.clone();
    gesture.connect_pressed(move |_, _, _, _| popover.popup());
    card.add_controller(gesture);
    card.append(&menu);
    NoteCard {
        widget: card,
        menu,
        action_buttons,
    }
}

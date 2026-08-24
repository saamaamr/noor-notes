use adw::prelude::*;

use super::adaptive_layout::editor_content_width;

pub fn configure_editor_canvas(editor: &gtk::TextView, rich_mode: bool) {
    let (horizontal, top, bottom) = if rich_mode { (8, 5, 5) } else { (16, 16, 24) };
    editor.set_left_margin(horizontal);
    editor.set_right_margin(horizontal);
    editor.set_top_margin(top);
    editor.set_bottom_margin(bottom);
    editor.add_css_class("nn-writing-canvas");
    if rich_mode {
        editor.set_pixels_above_lines(2);
        editor.set_pixels_below_lines(2);
        editor.set_pixels_inside_wrap(1);
        editor.add_css_class("nn-rich-writing-canvas");
        editor.remove_css_class("nn-source-canvas");
    } else {
        editor.set_pixels_above_lines(0);
        editor.set_pixels_below_lines(0);
        editor.set_pixels_inside_wrap(0);
        editor.add_css_class("nn-source-canvas");
        editor.remove_css_class("nn-rich-writing-canvas");
    }
    editor.set_tooltip_text(None);
    editor.update_property(&[gtk::accessible::Property::Label("Note body")]);
}

pub fn build_editor_canvas(editor: &gtk::TextView, rich_mode: bool) -> gtk::Widget {
    if !rich_mode {
        return editor.clone().upcast();
    }

    let clamp = adw::Clamp::builder()
        .maximum_size(860)
        .tightening_threshold(760)
        .child(editor)
        .build();
    clamp.add_css_class("nn-writing-clamp");
    clamp.set_hexpand(true);
    clamp.set_vexpand(true);
    clamp.upcast()
}

pub fn set_editor_canvas_available_width(canvas: &gtk::Widget, available_width: i32) {
    let Some(clamp) = canvas.downcast_ref::<adw::Clamp>() else {
        return;
    };
    let content_width = editor_content_width(available_width).max(1);
    clamp.set_maximum_size(content_width);
    clamp.set_tightening_threshold((content_width * 85 + 50) / 100);
}

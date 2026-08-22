use adw::prelude::*;

pub fn themed_popover(child: &impl IsA<gtk::Widget>) -> gtk::Popover {
    let popover = gtk::Popover::builder().child(child).build();
    style_popover(&popover);
    popover
}

pub fn style_popover(popover: &gtk::Popover) {
    popover.add_css_class("nn-menu-surface");
}

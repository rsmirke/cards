//! Drop-downs over a list of names.

use gtk::prelude::*;

/// Shows `labels` in `dropdown`, as plain text (they are the user's deck and
/// tag names), and makes the search in its popup work: `GtkDropDown` searches
/// by `expression`, which a model set by hand does not have
/// (`GtkDropDown:enable-search`).
pub fn set_labels(dropdown: &gtk::DropDown, labels: &[String]) {
    let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    dropdown.set_model(Some(&gtk::StringList::new(&refs)));
    dropdown.set_expression(Some(gtk::PropertyExpression::new(
        gtk::StringObject::static_type(),
        None::<gtk::Expression>,
        "string",
    )));
}

/// Lets `dropdown` be narrower than its chosen entry: the button shows the
/// entry on one line ending in "…" when it does not fit, and the popup
/// keeps every entry whole, with a check mark on the chosen one as
/// `GtkDropDown`'s own factory has. That factory makes a label that does
/// not ellipsize, so the button would be at least as wide as the entry
/// (gtkdropdown.c, `setup_item`); `factory` is used for the button and
/// `list-factory` for the popup (`GtkDropDown:list-factory` docs). The
/// entries must be `GtkStringObject`s (a `GtkStringList`, as [`set_labels`]
/// makes). Plain text, never markup.
pub fn shrinkable(dropdown: &gtk::DropDown) {
    dropdown.set_factory(Some(&string_factory(false)));
    dropdown.set_list_factory(Some(&string_factory(true)));
}

/// Labels of `GtkStringObject`s: ellipsized in the button, whole and with
/// a check mark on the chosen entry in the popup.
fn string_factory(popup: bool) -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, item| {
        let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let label = gtk::Label::builder()
            .xalign(0.0)
            .use_markup(false)
            .single_line_mode(true)
            .build();
        if !popup {
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            item.set_child(Some(&label));
            return;
        }
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        label.set_hexpand(true);
        row.append(&label);
        // As GTK's own: shown by opacity, so the popup keeps its width.
        let check = gtk::Image::builder()
            .icon_name("object-select-symbolic")
            .accessible_role(gtk::AccessibleRole::Presentation)
            .build();
        item.bind_property("selected", &check, "opacity")
            .transform_to(|_, selected: bool| Some(if selected { 1.0_f64 } else { 0.0 }))
            .sync_create()
            .build();
        row.append(&check);
        item.set_child(Some(&row));
    });
    factory.connect_bind(|_, item| {
        let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let label = item.child().and_then(|child| {
            child
                .clone()
                .downcast::<gtk::Label>()
                .ok()
                .or_else(|| child.first_child().and_downcast::<gtk::Label>())
        });
        if let (Some(string), Some(label)) =
            (item.item().and_downcast::<gtk::StringObject>(), label)
        {
            label.set_label(&string.string());
        }
    });
    factory
}

//! Drop-downs over a list of names.

use gtk::glib;
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
    dropdown.set_factory(Some(&button_factory()));
    dropdown.set_list_factory(Some(&popup_factory(dropdown)));
}

/// A plain label for a `GtkStringObject`, on one line.
fn string_label() -> gtk::Label {
    gtk::Label::builder()
        .xalign(0.0)
        .use_markup(false)
        .single_line_mode(true)
        .build()
}

fn bind_label(item: &gtk::ListItem, label: &gtk::Label) {
    if let Some(string) = item.item().and_downcast::<gtk::StringObject>() {
        label.set_label(&string.string());
    }
}

/// The button: the chosen entry, ellipsized.
fn button_factory() -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let label = string_label();
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        item.set_child(Some(&label));
    });
    factory.connect_bind(|_, item| {
        let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        if let Some(label) = item.child().and_downcast::<gtk::Label>() {
            bind_label(item, &label);
        }
    });
    factory
}

/// Whether the popup's check mark shows on `item`: it is the drop-down's
/// chosen item. Compared by item, as GTK's own factory does
/// (gtkdropdown.c, `selected_item_changed`), not by `GtkListItem:selected`,
/// which follows the pointer and the arrow keys in the popup, nor by
/// position, which a search in the popup changes.
fn is_chosen(dropdown: &gtk::DropDown, item: &gtk::ListItem) -> bool {
    same_item(dropdown.selected_item().as_ref(), item.item().as_ref())
}

/// Whether `entry` is the `chosen` item (none is never chosen).
fn same_item(chosen: Option<&glib::Object>, entry: Option<&glib::Object>) -> bool {
    entry.is_some() && chosen == entry
}

/// The popup: every entry whole, a check mark on the chosen one (shown by
/// opacity, as GTK's, so the popup keeps its width).
fn popup_factory(dropdown: &gtk::DropDown) -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();
    let weak = dropdown.downgrade();
    factory.connect_setup(move |_, item| {
        let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let label = string_label();
        label.set_hexpand(true);
        row.append(&label);
        let check = gtk::Image::builder()
            .icon_name("object-select-symbolic")
            .accessible_role(gtk::AccessibleRole::Presentation)
            .opacity(0.0)
            .build();
        row.append(&check);
        item.set_child(Some(&row));
        // The choice can change while the popup is open (Enter, a click);
        // the handler holds weak references only.
        if let Some(dropdown) = weak.upgrade() {
            let item = item.downgrade();
            let check = check.downgrade();
            dropdown.connect_selected_item_notify(move |dropdown| {
                if let (Some(item), Some(check)) = (item.upgrade(), check.upgrade()) {
                    check.set_opacity(if is_chosen(dropdown, &item) { 1.0 } else { 0.0 });
                }
            });
        }
    });
    let weak = dropdown.downgrade();
    factory.connect_bind(move |_, item| {
        let Some(item) = item.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let Some(row) = item.child() else {
            return;
        };
        if let Some(label) = row.first_child().and_downcast::<gtk::Label>() {
            bind_label(item, &label);
        }
        if let (Some(check), Some(dropdown)) = (row.last_child(), weak.upgrade()) {
            check.set_opacity(if is_chosen(&dropdown, item) { 1.0 } else { 0.0 });
        }
    });
    factory
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The check mark is on the drop-down's chosen item, wherever it is in
    /// the popup, and on nothing when nothing is chosen.
    #[test]
    fn the_check_mark_is_on_the_chosen_entry() {
        crate::testing::with_widgets(|| {
            let dropdown = gtk::DropDown::builder().build();
            set_labels(
                &dropdown,
                &["All".to_owned(), "a".to_owned(), "b".to_owned()],
            );
            let model = dropdown.model().expect("set_labels sets a model");
            dropdown.set_selected(2);
            let chosen = dropdown.selected_item();
            assert!(same_item(chosen.as_ref(), model.item(2).as_ref()));
            assert!(!same_item(chosen.as_ref(), model.item(1).as_ref()));
            assert!(!same_item(None, model.item(1).as_ref()));
            assert!(!same_item(None, None));
        });
    }
}

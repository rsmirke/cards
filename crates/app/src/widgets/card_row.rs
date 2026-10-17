//! One row of the deck page's card list (ADR 0042): the front as the title,
//! the back dimmed under it with the tags as pills at the end of that line,
//! an icon when the card is suspended, and a menu with Edit, Card
//! Information, Suspend or Unsuspend, Reset Progress and Delete.
//! Layout in `data/card_row.blp`; what a screen reader says in `text.rs`.
//!
//! Rows are recycled by the `GtkListView`: [`CardRow::bind`] points a row
//! at another card, reusing its pill labels. The `compact` property (bound
//! to the page's, a narrow window) shows fewer pills. As in `DeckRow`, the
//! menu items activate the row's own `row.edit` / `row.info` /
//! `row.suspend` / `row.unsuspend` / `row.reset` / `row.delete` actions,
//! which forward to the page's `deck.edit-card` / `deck.card-info` /
//! `deck.suspend-card` / `deck.unsuspend-card` / `deck.reset-card` /
//! `deck.delete-card` with the bound card's id.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use super::CardObject;
use super::tag_pills::text::{COMPACT_ROW_PILLS, ROW_PILLS};
use crate::ui::rows;

pub mod text;

mod imp {
    use std::cell::{Cell, RefCell};

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::glib;

    use crate::widgets::TagPills;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::CardRow)]
    #[template(resource = "/io/github/rsmirke/Rappel/card_row.ui")]
    pub struct CardRow {
        #[template_child]
        pub front_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub detail_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub back_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tag_pills: TemplateChild<TagPills>,
        #[template_child]
        pub suspended_icon: TemplateChild<gtk::Image>,
        /// A narrow row: fewer pills, so the back keeps some room.
        #[property(get, set)]
        pub compact: Cell<bool>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub card_menu: TemplateChild<gtk::gio::MenuModel>,
        /// The bound card, empty while unbound.
        pub card_id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardRow {
        const NAME: &'static str = "RappelCardRow";
        type Type = super::CardRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            TagPills::ensure_type();
            klass.bind_template();
            klass.install_action("row.edit", None, |row, _, _| {
                row.forward("deck.edit-card");
            });
            klass.install_action("row.info", None, |row, _, _| {
                row.forward("deck.card-info");
            });
            klass.install_action("row.suspend", None, |row, _, _| {
                row.forward("deck.suspend-card");
            });
            klass.install_action("row.unsuspend", None, |row, _, _| {
                row.forward("deck.unsuspend-card");
            });
            klass.install_action("row.reset", None, |row, _, _| {
                row.forward("deck.reset-card");
            });
            klass.install_action("row.delete", None, |row, _, _| {
                row.forward("deck.delete-card");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CardRow {
        fn constructed(&self) {
            self.parent_constructed();
            crate::ui::rows::menu_on_demand(&self.menu_button, &self.card_menu);
            let row = self.obj();
            row.connect_compact_notify(super::CardRow::update_pill_count);
            row.update_pill_count();
        }
    }
    impl WidgetImpl for CardRow {}
    impl BoxImpl for CardRow {}
}

glib::wrapper! {
    pub struct CardRow(ObjectSubclass<imp::CardRow>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for CardRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl CardRow {
    /// Shows `item` in this row. Called on every bind of a recycled row,
    /// so it only sets texts and reuses the pills (`TagPills::set_tags`).
    pub fn bind(&self, item: &CardObject) {
        let imp = self.imp();
        let front = item.front_preview();
        imp.front_label.set_label(front);
        let back = item.back_preview();
        imp.back_label.set_label(back);
        let tags = item.tags();
        imp.tag_pills.set_tags(&tags);
        imp.tag_pills.set_visible(!tags.is_empty());
        // Without a back or tags, the row is its front alone.
        imp.detail_box
            .set_visible(!back.is_empty() || !tags.is_empty());
        rows::label_menu_button(&imp.menu_button, front);
        let suspended = item.card().suspended;
        imp.suspended_icon.set_visible(suspended);
        // The row offers the one of the two that changes something.
        self.action_set_enabled("row.suspend", !suspended);
        self.action_set_enabled("row.unsuspend", suspended);
        imp.card_id.replace(item.card().id.clone());
    }

    fn update_pill_count(&self) {
        let max = if self.compact() {
            COMPACT_ROW_PILLS
        } else {
            ROW_PILLS
        };
        self.imp().tag_pills.set_max_shown(max);
    }

    /// Forgets the card, before the row is recycled.
    pub fn unbind(&self) {
        self.imp().card_id.borrow_mut().clear();
    }

    fn forward(&self, action: &str) {
        let id = self.imp().card_id.borrow().clone();
        if id.is_empty() {
            return;
        }
        rows::forward_id(self, action, &id);
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use rappel_core::domain::{Card, Tag, TaggedCard};

    use super::*;

    /// Card `i` of a deck: a Markdown front, a back and a few tags.
    fn card(i: usize) -> CardObject {
        let tags = (0..i % 6)
            .map(|t| Tag {
                id: format!("tag-{t}"),
                name: format!("tag number {t}"),
            })
            .collect();
        CardObject::new(TaggedCard {
            card: Card {
                id: format!("card-{i:05}"),
                deck_id: "deck".to_owned(),
                front_md: format!("**Card {i}**: what is $\\frac{{{i}}}{{2}}$?"),
                back_md: format!("Answer {i}, with `code` and a [link](x)"),
                created_at: 0,
                updated_at: 0,
                suspended: i.is_multiple_of(7),
            },
            tags,
        })
    }

    #[test]
    fn a_row_shows_the_card_and_reads_it_out() {
        crate::testing::with_widgets(|| {
            let row = CardRow::default();
            let item = card(4);
            row.bind(&item);
            let imp = row.imp();
            assert_eq!(imp.front_label.label(), "Card 4: what is \\frac{4}{2}?");
            assert_eq!(imp.back_label.label(), "Answer 4, with code and a link");
            assert_eq!(imp.tag_pills.tags().len(), 4);
            assert!(!imp.suspended_icon.is_visible());
            // The menu is built when it first opens, not with the row.
            assert!(imp.menu_button.popover().is_none());
            assert_eq!(
                item.row_label(),
                "Card 4: what is \\frac{4}{2}?, back: Answer 4, with code and a link, \
                 tags: tag number 0, tag number 1, tag number 2, tag number 3"
            );
            // A recycled row follows its new card.
            row.bind(&card(42));
            assert!(imp.suspended_icon.is_visible());
            assert!(!imp.tag_pills.is_visible());
        });
    }

    /// Scrolling binds recycled rows over and over: binding one row to each
    /// of 5,000 cards (making each preview the first time) must stay fast.
    #[test]
    fn binding_a_row_to_5000_cards_is_fast() {
        let elapsed = crate::testing::with_widgets(|| {
            let items: Vec<CardObject> = (0..5_000).map(card).collect();
            let row = CardRow::default();
            let start = Instant::now();
            for item in &items {
                row.bind(item);
                drop(item.row_label());
                row.unbind();
            }
            start.elapsed()
        });
        crate::testing::print_timing("CardRow::bind, 5,000 cards", elapsed);
        // About 115 ms in a debug build (30 ms of it binding, the rest making
        // each card's two previews once); the bound leaves 10 times that.
        assert!(elapsed < Duration::from_millis(1_200), "{elapsed:?}");
    }
}

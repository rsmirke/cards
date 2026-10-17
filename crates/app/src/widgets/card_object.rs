//! `CardObject`: one card as a list model item, for the deck page's
//! `GtkListView` (ADR 0013). It holds the card and its one-line preview
//! (plain text, never markup), computed once, so binding a recycled row is
//! cheap.

use std::cell::{Cell, Ref, RefCell};

use gtk::glib;
use gtk::subclass::prelude::*;
use rappel_core::content::plain_summary;
use rappel_core::domain::Card;

mod imp {
    use super::{Card, Cell, RefCell};
    use gtk::glib;
    use gtk::subclass::prelude::*;

    #[derive(Debug)]
    pub struct CardObject {
        pub card: RefCell<Card>,
        pub preview: RefCell<String>,
        pub is_new: Cell<bool>,
    }

    /// `GObject` needs a default; [`super::CardObject::new`], the only
    /// constructor, replaces this empty card at once.
    impl Default for CardObject {
        fn default() -> Self {
            Self {
                card: RefCell::new(Card {
                    id: String::new(),
                    deck_id: String::new(),
                    front_md: String::new(),
                    back_md: String::new(),
                    created_at: 0,
                    updated_at: 0,
                }),
                preview: RefCell::new(String::new()),
                is_new: Cell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardObject {
        const NAME: &'static str = "RappelCardObject";
        type Type = super::CardObject;
    }

    impl ObjectImpl for CardObject {}
}

glib::wrapper! {
    pub struct CardObject(ObjectSubclass<imp::CardObject>);
}

impl CardObject {
    pub fn new(card: Card) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().preview.replace(plain_summary(&card.front_md));
        obj.imp().card.replace(card);
        obj
    }

    /// The card. An edit replaces the whole object in the model.
    pub fn card(&self) -> Ref<'_, Card> {
        self.imp().card.borrow()
    }

    /// Whether the card is new: no surviving review since it was created or
    /// last reset. Decides whether its row offers Reset Progress. A card
    /// starts as not new; the page sets it when it puts the card in the
    /// list, and replaces the object when it changes, so the row rebinds.
    pub fn is_new(&self) -> bool {
        self.imp().is_new.get()
    }

    /// Marks the card new or not, before the object is put in the list.
    #[must_use]
    pub fn with_new(self, is_new: bool) -> Self {
        self.imp().is_new.set(is_new);
        self
    }

    /// The front on one line, without Markdown syntax (see
    /// [`plain_summary`]).
    pub fn preview(&self) -> Ref<'_, String> {
        self.imp().preview.borrow()
    }
}

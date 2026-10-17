# 44. The All Cards page's layout: search header, compact filters, shared rows, selection mode

- Status: Accepted
- Date: 2026-10-17
- Amends: ADR 0036 (the search field's key capture, the filter bar, the rows, the selection-mode header, the empty states)
- Refers to: ADR 0040 (shell), ADR 0042 (card rows and tag pills), `docs/briefs/phase-1-finish.md` ("Browser", "Tags", "Design rules"), GNOME HIG "Selection mode"

## Context

The phase 1 brief asks for the All Cards page: a search bar in the GNOME style, filters as compact controls that collapse at 360 px, rows consistent with the deck page's card list (ADR 0042), a selection mode with a bottom action bar that follows the HIG, and proper empty and no-results states. Before this, the three filter drop-downs stacked under 500 sp and took most of a 360×294 window, the rows had their own layout (with the tags as a dimmed text caption, and an odd indent from `rich-list` inside a 900 px clamp), and the action bar carried the "3 selected" count.

## Decision

### Search stays the header's title widget

A `GtkSearchEntry` in an `AdwClamp` (400 px) as the `AdwHeaderBar`'s title widget, always shown, not a `GtkSearchBar` under the header. A search bar is for a page where search is occasional and revealed on demand (Files, Settings' list); All Cards exists to search, so its field is always there and has the keyboard when the page opens. It costs no height, which matters at 294 px.

**Typing anywhere on the page searches.** ADR 0036 left it out because the `key-capture-widget` *property* is GTK 4.22; the method `gtk_search_entry_set_key_capture_widget` is GTK 4.0 (the GIR has no `version` on the method, only on the property), so it is used, with the page as the capture widget. GTK takes the keys in the bubble phase, so the list's own keys (Ctrl+A, Space, arrows) and the drop-downs' search fields come first. The captured text goes into the field without moving the focus, so the page then moves the focus into it (`grab_focus_without_selecting`) so Backspace and arrows act on the text. Space on a row is taken by the list (it chooses in selection mode, does nothing otherwise) so it never starts a search with a space. While cards are being chosen the capture is off: the field is not shown then.

Ctrl+F still focuses the field; in selection mode it does nothing (Cancel or Escape first), so a shortcut never drops a choice of cards silently.

### Filters on one line, always

Deck, tag and state stay `GtkDropDown`s (the first two with a search field), side by side in one row under the header, in a 600 px clamp so they line up with the list, with the card count ("336 cards", `.numeric`, dimmed) at the end. They share the width equally and never wrap or stack: at 360 px each is about 105 px, and the chosen entry ends in "…" if it does not fit (`ui::dropdown::shrinkable`: GTK's default factory makes a label that cannot ellipsize; the popup keeps whole names and GTK's check mark). Under 500 sp the count is hidden to leave the width to the filters; a screen reader still hears it after each search. The bar is 46 px high at every width.

Not a filter button with a popover: a hidden filter silently narrows the list (ADR 0036), and three drop-downs fit on one line even on a phone. Not an `AdwToggleGroup` for the state: five values (Any, New, Learning, Due, Suspended) do not fit 360 px as toggles.

### One row for both lists

The All Cards rows are the deck page's `CardRow` (ADR 0042), not a copy: `CardRow::for_browser()` makes a row whose menu forwards to `browser.*` instead of `deck.*` (the action names are the same), whose third line (`caption_label`, `.caption` `.dimmed`) says the deck and status ("Maths · Due in 3 days"), and which has a long-press gesture (touch only) that starts choosing. A check button before the text shows while `selecting` is on, in place of the ⋮ menu. `bind_browse` binds a `BrowseItem` and follows its `selected`; the list item's accessible label is "front, back: …, tags: …, deck: Maths, Due in 3 days[, selected / not selected]" (`browser_text::row_label` around `card_row::text::row_label`). `BrowserRow` and `data/browser_row.blp` are gone.

So the rows look and cost the same: two lines like the deck page's (front; back with the tag pills), plus the deck and status line; the pause icon for a suspended card; the menu made on first use (`ui::rows::menu_on_demand`); the pills relabelled, never remade. The list is clamped to 600 px with `separators` and without `rich-list`, as on the deck page, which removes the old indent. The narrow rows show one pill, as on the deck page; they keep the back (the old compact rows dropped it), so a row is the same at every width. A test binds one row to 4,000 of 5,000 `BrowseItem`s and checks that no pill label or popover is made and only those cards get previews (about 120 ms CPU in a debug build, bounded at 2 s as for the deck rows).

### Selection mode as the HIG describes it

The HIG: "The header bar should show a cancel button and the heading should indicate the number of selected items", and the actions go in an action bar at the bottom. So in selection mode:

- the header's title is an `AdwWindowTitle` "3 Selected" ("Select Cards" before the first), in place of the search field (a `GtkStack` as the title widget);
- **Cancel** (a text button) replaces the Select button at the end, and **Select All** (icon and label; the icon alone under 500 sp, with its tooltip and accessible label) comes at the start;
- the page's `can-pop` is off, which hides Back and keeps libadwaita's Escape and swipe from popping the page (`AdwNavigationPage:can-pop` docs), so Escape leaves selection mode, as ADR 0036 meant (its consequence about `pop-on-escape` is settled);
- the bottom `GtkActionBar` has Move…, Tags and Suspend at the start, Reset and Delete at the end, away from the others; under 500 sp they keep only their icons. The count moved from the action bar to the header.

A header button that hides while it has the keyboard (Cancel, Select All) hands the focus to the Select button that comes back; Select moves it to the list. The filters stay usable while choosing; as before, a card that leaves the list is no longer chosen.

Not done: the HIG's check box on hover in normal mode. The list's click opens the editor (ADR 0036), the Select button, Ctrl+A and a long press already start choosing, and a hover-only control would fail the phone rule (SPEC §4.8).

### Empty and no-results states

One `AdwStatusPage` whose icon, title, sentence and button come from `browser_text::empty_page`, `GO_TO_DECKS` and `clear_label` (unit-tested):

- no card at all: "No Cards Yet", "Cards you add to your decks will show up here", **Go to Decks** (back to the deck list, where decks are made and cards added);
- nothing matches: "No Results Found", "Try a different search or other filters", and a button that says what it clears: **Clear Search**, **Clear Filters** or **Clear Search and Filters**.

Both buttons are `.pill .suggested-action` (brief, design rules). Escape in an empty search field moves to the list or to this button.

## Consequences

- `CardRow` serves two pages. A change to it shows on both; the deck page's rows never show the check button or the third line.
- The page's minimum size is the window's (360×294), like the deck page's; at 294 px about two rows show under the filters, or one and a half with the action bar.
- At 360 px a long deck name in the deck filter is cut with "…"; its full name is in the popup.
- Typing while the focus is on a header or action-bar button also searches (it is on the page); that is the GNOME behaviour of a key-capture search.

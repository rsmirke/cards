//! Statistics and card information from the database (ADR 0032).

use rusqlite::{Connection, params};

use super::Store;
use super::cards::live_card;
use super::decks::ensure_live_deck;
use super::review::counted_event;
use super::schedule::{read_log, to_f32};
use super::settings::scheduler_in;
use crate::error::Result;
use crate::stats::{CardInfo, CountedReview, DayGrid, ScheduledCard, Stats, card_info};

/// The cards a [`Store::stats`] call covers. Deleted cards and decks never
/// count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope<'a> {
    /// Every live deck.
    All,
    /// One live deck.
    Deck(&'a str),
}

impl Scope<'_> {
    fn deck(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Deck(id) => Some(id),
        }
    }
}

impl Store {
    /// The statistics of `scope` at `now` (UTC milliseconds), in one read
    /// transaction. Reviews per day, time and pass rate cover the days of
    /// `past`; the forecast covers the days of `ahead` (day 0 takes the
    /// overdue cards too). The expected retention is the mean recall
    /// probability of every live scheduled card at `now`. All counts use
    /// surviving entries only (ADR 0028, 0032).
    ///
    /// # Errors
    /// [`Error::DeckNotFound`](crate::Error::DeckNotFound) for a missing or
    /// deleted deck; SQLite errors.
    pub fn stats(
        &self,
        scope: Scope<'_>,
        now: i64,
        past: &DayGrid,
        ahead: &DayGrid,
    ) -> Result<Stats> {
        let tx = self.conn.unchecked_transaction()?;
        if let Scope::Deck(id) = scope {
            ensure_live_deck(&tx, id)?;
        }
        let reviews = counted_in_window(&tx, scope.deck(), past.start(), past.end())?;
        let cards = scheduled_cards(&tx, scope.deck())?;
        // Read only: ending it commits nothing.
        tx.finish()?;
        Ok(Stats::compute(past, ahead, now, reviews, cards))
    }

    /// The properties and history of card `card_id` at `now`, replayed from
    /// its log in one read transaction (issue #39, ADR 0032).
    ///
    /// # Errors
    /// [`Error::CardNotFound`](crate::Error::CardNotFound) for a missing or
    /// deleted card; scheduler and SQLite errors.
    pub fn card_info(&self, card_id: &str, now: i64) -> Result<CardInfo> {
        let tx = self.conn.unchecked_transaction()?;
        let card = live_card(&tx, card_id)?;
        let log = read_log(&tx, card_id)?;
        let scheduler = scheduler_in(&tx)?;
        tx.finish()?;
        card_info(&scheduler, &card, &log, now)
    }
}

/// The counted reviews of live cards in live decks (`?1`: the deck, or NULL
/// for all) made in `[?2, ?3)`.
const COUNTED_IN_WINDOW: &str = concat!(
    "SELECT e.reviewed_at, e.grade = 'pass', e.duration_ms
     FROM review_events e
     JOIN cards c ON c.id = e.card_id JOIN decks d ON d.id = c.deck_id
     WHERE c.deleted_at IS NULL AND d.deleted_at IS NULL
       AND (?1 IS NULL OR c.deck_id = ?1)
       AND e.reviewed_at >= ?2 AND e.reviewed_at < ?3 AND ",
    counted_event!()
);

/// The counted reviews of live cards in live decks (of `deck`, if given)
/// made in `[from, to)`.
fn counted_in_window(
    conn: &Connection,
    deck: Option<&str>,
    from: i64,
    to: i64,
) -> Result<Vec<CountedReview>> {
    let mut stmt = conn.prepare(COUNTED_IN_WINDOW)?;
    let rows = stmt.query_map(params![deck, from, to], |r| {
        Ok(CountedReview {
            at: r.get(0)?,
            passed: r.get(1)?,
            duration_ms: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The schedules of live cards in live decks (of `deck`, if given).
fn scheduled_cards(conn: &Connection, deck: Option<&str>) -> Result<Vec<ScheduledCard>> {
    let mut stmt = conn.prepare(
        "SELECT s.due_at, s.stability, s.difficulty, s.last_review_at
         FROM card_schedule s
         JOIN cards c ON c.id = s.card_id JOIN decks d ON d.id = c.deck_id
         WHERE c.deleted_at IS NULL AND d.deleted_at IS NULL
           AND (?1 IS NULL OR c.deck_id = ?1)",
    )?;
    let rows = stmt.query_map([deck], |r| {
        Ok(ScheduledCard {
            due_at: r.get(0)?,
            stability: to_f32(r.get(1)?),
            difficulty: to_f32(r.get(2)?),
            last_review_at: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
mod tests;

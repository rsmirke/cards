# 32. Statistics and card information in core

- Status: Accepted
- Date: 2026-10-17
- Amends: SPEC §4.7 (time studied per day and the card information are listed), ADR 0028 (the "counted" rule is shared with the statistics)
- Refers to: #39 (card information screen)

## Context

SPEC §4.7 wants reviews per day, pass rate, a due forecast and the expected retention, per deck and global, as pure functions in `core::stats`. Issue #39 wants a card's properties and history. ADR 0028 left a rule for them: count surviving entries only. Four things needed a decision: what counts, how days are cut without a time zone in core, which cards are covered, and how to stay fast at 5,000 cards and 100,000 reviews.

## Decision

**What counts.** A *counted review* is a review that no undo cancels, made after the card's last reset that no undo cancels: the rule the daily limits already use. `stats::counted_reviews` states it for one card's log; the SQL (`counted_event!` in `storage/review.rs`) states it for the database, and the limits and the statistics share that one text. Property tests check the SQL against the pure function.

**Days.** Core knows no time zones (ADR 0006, 0014). The caller passes a `DayGrid`: strictly increasing UTC timestamps at which consecutive study days begin (n + 1 bounds for n days), which the app computes with `clock.rs` (GLib, already tested across daylight-saving days) from the stored day start hour. A 23- or 25-hour day is only a shorter or longer bucket, so core needs no calendar code. A grid with fewer than two bounds, or bounds not strictly increasing, is `Error::InvalidDayGrid`. The app passes 31 bounds for the last 30 days (today last) and 15 for the next 14 (today first).

**The figures.**
- Reviews, passes and time per day: counted reviews in the past grid. Time is the sum of the recorded durations (already capped at 60 s).
- Pass rate: passes over reviews in the past grid, all phases, 10-minute steps included; `None` without reviews.
- Due forecast: live scheduled cards by the day their `due_at` falls on, from the cache. Day 0 also takes every overdue card; `overdue` says how many were due before the grid began.
- Expected retention: the mean over live scheduled cards of `fsrs::current_retrievability` with `FSRS6_DEFAULT_DECAY`, the decay of the default parameters the scheduler uses (a test checks that recall after the scheduler's own interval equals the desired retention). Elapsed time is a fraction of days (to the second) since the last review, not the scheduler's rounded whole days: it is a value for this moment. `None` without scheduled cards.

**Which cards.** Live cards in live decks only, for every figure and every scope (`Scope::All` or `Scope::Deck`). Deleting a card or deck takes it out of the statistics, and the per-deck figures add up to the global ones exactly (the mean retention through `scheduled_cards`).

**Card information.** `Store::card_info(card, now)` replays the surviving log with `Scheduler::schedule`, as the cache is rebuilt, and returns the dates, the counted reviews, passes and lapses, stability, difficulty, retrievability now and next due since the last surviving reset, and the history: every surviving review (time, grade, duration, the interval it gave) and reset (a marker), oldest first. Undone entries are left out. Reviews before a reset stay in the history; they are not counted.

**SQL and Rust.** SQL picks the counted reviews of the window (one indexed scan) and the live schedules; Rust buckets them. That is simpler than a grid table in SQL and fast enough.

**Migration 0009.** The counted rule asks, for each review, whether a surviving reset came after it. Without help SQLite reads every later entry of the card to find out. A partial index `review_events_resets(card_id, reviewed_at, id) WHERE kind = 'reset'` holds only the rare resets. At 5,000 cards and 100,000 reviews the global statistics went from 1.3 s to 0.28 s in a debug build; the daily limits use the same condition and gain too. No data changes.

## Consequences

- The UI slice only builds two grids from `clock.rs` and draws `Stats`; it does no counting.
- Timings at 5,000 cards and about 100,000 reviews, debug build: global 0.28 s, one deck 0.20 s, `card_info` of a card with 2,000 reviews 9 ms. The tests bound them at about 10 times that (ADR 0025).
- A card's information is replayed on demand, so it cannot disagree with the cache; a property test checks it does not.
- The pass rate counts every answer, so a card in its 10-minute step weighs as much as a review. A "true retention" figure (review-phase answers only) can be added later without changing the log.

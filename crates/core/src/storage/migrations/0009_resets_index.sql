-- Migration 0009: an index on resets (ADR 0032). NEVER EDIT THIS FILE; add
-- a new migration.
--
-- The daily limits and the statistics ask, for each review, whether a
-- reset that no undo cancels came after it (ADR 0028). Resets are rare,
-- so a partial index holds only them and answers from the index alone,
-- instead of reading every later entry of the card. Nothing else changes.

CREATE INDEX review_events_resets ON review_events(card_id, reviewed_at, id) WHERE kind = 'reset';

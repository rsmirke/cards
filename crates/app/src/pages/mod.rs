//! Full-window pages. Each is a `.rs` file with its layout in `data/*.blp`.

mod decks_page;
mod recovery_page;

pub use decks_page::DecksPage;
pub use recovery_page::RecoveryPage;

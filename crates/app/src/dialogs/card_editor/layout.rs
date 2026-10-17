//! Which of the card editor's panes are shown, decided without GTK (ADR
//! 0043). Wide, the text and the preview are side by side. Narrow (the
//! dialog's breakpoint), one of them fills the dialog, and the Edit /
//! Preview toggle in the bottom bar picks which.

/// What the narrow layout shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Edit,
    Preview,
}

impl Mode {
    /// The `AdwToggle` name of each mode, in `data/card_editor.blp`.
    pub const EDIT: &'static str = "edit";
    pub const PREVIEW: &'static str = "preview";

    /// The mode of the toggle named `name`; Edit for any other.
    pub fn from_name(name: Option<&str>) -> Self {
        if name == Some(Self::PREVIEW) {
            Self::Preview
        } else {
            Self::Edit
        }
    }
}

/// The panes on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panes {
    /// The text fields.
    pub edit: bool,
    /// The preview, which is rendered only while shown.
    pub preview: bool,
    /// The line between them.
    pub separator: bool,
}

/// The panes for a `narrow` dialog or a wide one, in `mode`. Wide shows
/// both whatever the mode, so a dialog that grows wide never hides the text.
pub fn panes(narrow: bool, mode: Mode) -> Panes {
    if narrow {
        Panes {
            edit: mode == Mode::Edit,
            preview: mode == Mode::Preview,
            separator: false,
        }
    } else {
        Panes {
            edit: true,
            preview: true,
            separator: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_shows_both_side_by_side() {
        for mode in [Mode::Edit, Mode::Preview] {
            assert_eq!(
                panes(false, mode),
                Panes {
                    edit: true,
                    preview: true,
                    separator: true
                }
            );
        }
    }

    #[test]
    fn narrow_shows_the_one_the_toggle_picks() {
        let edit = panes(true, Mode::Edit);
        assert!(edit.edit && !edit.preview && !edit.separator);
        let preview = panes(true, Mode::Preview);
        assert!(!preview.edit && preview.preview && !preview.separator);
    }

    #[test]
    fn modes_are_named_as_the_toggles() {
        assert_eq!(Mode::from_name(Some("preview")), Mode::Preview);
        assert_eq!(Mode::from_name(Some("edit")), Mode::Edit);
        assert_eq!(Mode::from_name(None), Mode::Edit);
    }
}

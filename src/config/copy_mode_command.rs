use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CopyModeCommand {
    CursorLeft,
    CursorDown,
    CursorUp,
    CursorRight,
    StartOfLine,
    EndOfLine,
    BackToIndentation,
    HistoryTop,
    HistoryBottom,
    PageUp,
    PageDown,
    HalfpageUp,
    HalfpageDown,
    NextWord,
    PreviousWord,
    NextWordEnd,
    PreviousParagraph,
    NextParagraph,
    BeginSelection,
    SelectLine,
    CopySelection,
    Cancel,
    ClearSelectionOrCancel,
    SearchForward,
    SearchBackward,
    SearchAgain,
    SearchReverse,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CopyModeCommandSpec {
    Known(CopyModeCommand),
    Unknown(String),
}

impl CopyModeCommand {
    pub fn is_repeatable(self) -> bool {
        matches!(
            self,
            Self::CursorLeft
                | Self::CursorDown
                | Self::CursorUp
                | Self::CursorRight
                | Self::PageUp
                | Self::PageDown
                | Self::HalfpageUp
                | Self::HalfpageDown
                | Self::NextWord
                | Self::PreviousWord
                | Self::NextWordEnd
                | Self::PreviousParagraph
                | Self::NextParagraph
                | Self::SearchAgain
                | Self::SearchReverse
        )
    }
}

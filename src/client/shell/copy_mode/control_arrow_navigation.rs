use crossterm::event::{KeyCode, KeyModifiers};

const LINE_JUMP_ROWS: i16 = 7;

pub(super) enum ControlArrowNavigation {
    MoveRows(i16),
    MoveWord(crate::api::schema::PaneCopyMotion),
}

pub(super) fn resolve(key: &crate::input::TerminalKey) -> Option<ControlArrowNavigation> {
    match (key.code, key.modifiers) {
        (KeyCode::Up, modifiers) if modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
            Some(ControlArrowNavigation::MoveRows(-LINE_JUMP_ROWS))
        }
        (KeyCode::Down, modifiers) if modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT => {
            Some(ControlArrowNavigation::MoveRows(LINE_JUMP_ROWS))
        }
        (KeyCode::Left, KeyModifiers::CONTROL) => Some(ControlArrowNavigation::MoveWord(
            crate::api::schema::PaneCopyMotion::PreviousWordStart,
        )),
        (KeyCode::Right, KeyModifiers::CONTROL) => Some(ControlArrowNavigation::MoveWord(
            crate::api::schema::PaneCopyMotion::NextWordStart,
        )),
        _ => None,
    }
}

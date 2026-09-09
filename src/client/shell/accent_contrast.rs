use super::*;

pub(super) fn accent_contrast_fg(palette: &Palette) -> ratatui::style::Color {
    let ratatui::style::Color::Rgb(red, green, blue) = palette.accent else {
        return panel_contrast_fg(palette);
    };
    match (crate::terminal_theme::RgbColor {
        r: red,
        g: green,
        b: blue,
    })
    .inferred_appearance()
    {
        crate::terminal_theme::HostAppearance::Light => ratatui::style::Color::Black,
        crate::terminal_theme::HostAppearance::Dark => ratatui::style::Color::White,
    }
}

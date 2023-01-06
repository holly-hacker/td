use tui::style::{Color, Modifier, Style};

/// The minimum width a modal window can be
pub const MIN_MODAL_WIDTH: u16 = 32;

pub const ACCENT_COLOR: Color = Color::Blue;

pub const STANDARD_STYLE_FG_WHITE: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const BOLD: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const BOLD_UNDERLINED: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::BOLD.union(Modifier::UNDERLINED),
    sub_modifier: Modifier::empty(),
};

/// The style for unselected list items
pub const LIST_STYLE: Style = Style {
    fg: Some(Color::Gray),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

/// The style for selected list items
pub const LIST_HIGHLIGHT_STYLE: Style = Style {
    fg: Some(Color::Black),
    bg: Some(ACCENT_COLOR),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

/// The style for unselected tabs
pub const TAB_STYLE: Style = Style {
    fg: Some(Color::DarkGray),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

/// The style for selected tabs
pub const TAB_HIGHLIGHT_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

/// The style for a textbox without background
pub const TEXTBOX_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

/// The style for a textbox with a background
pub const TEXTBOX_STYLE_BG: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::DarkGray),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
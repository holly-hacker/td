use tui::style::{Color, Modifier, Style};

/// The minimum width a modal window can be
pub const MIN_MODAL_WIDTH: u16 = 32;

pub const ACCENT_COLOR: Color = Color::LightBlue;

pub const FG_WHITE: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const FG_GREEN: Style = Style {
    fg: Some(Color::Green),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const FG_RED: Style = Style {
    fg: Some(Color::Red),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const FG_DIM: Style = Style {
    fg: Some(Color::DarkGray),
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

pub const ITALIC: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::ITALIC,
    sub_modifier: Modifier::empty(),
};

pub const STARTED_TASK: Style = Style {
    fg: Some(Color::Yellow),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const COMPLETED_TASK: Style = Style {
    fg: Some(Color::DarkGray),
    bg: None,
    add_modifier: Modifier::ITALIC.union(Modifier::CROSSED_OUT),
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

pub const KEYBINDS_TEXT_ACTIVE: Style = Style {
    fg: Some(Color::Gray),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const KEYBINDS_TEXT_INACTIVE: Style = Style {
    fg: Some(Color::DarkGray),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const KEYBINDS_CHAR_ACTIVE: Style = Style {
    fg: Some(ACCENT_COLOR),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

pub const KEYBINDS_CHAR_INACTIVE: Style = KEYBINDS_TEXT_INACTIVE;

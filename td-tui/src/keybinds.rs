use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub const KEYBIND_TASKPAGE_PANE_SETTINGS: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Right, "Select settings pane");
pub const KEYBIND_TASKPAGE_PANE_TASKS: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Left, "Select tasks pane");

pub const KEYBIND_TASK_MARK_STARTED: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Char(' '), "Mark as started");
pub const KEYBIND_TASK_MARK_DONE: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Enter, "Mark as done");
pub const KEYBIND_TASK_NEW: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('n'), "New task");
pub const KEYBIND_TASK_DELETE: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('x'), "Delete");
pub const KEYBIND_TASK_EDIT: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('e'), "Edit");
pub const KEYBIND_TASK_ADD_TAG: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('t'), "Add tag");
pub const KEYBIND_TASK_ADD_DEPENDENCY: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Char('d'), "Add dependency");
pub const KEYBIND_TASK_RENAME: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('r'), "Rename");

pub const KEYBIND_TABS_NEXT: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Tab, "Next tab");
pub const KEYBIND_TABS_PREV: &SimpleKeybind = &SimpleKeybind::new_hidden(KeyCode::BackTab);

pub const KEYBIND_CONTROLS_CHECKBOX_TOGGLE: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Char(' '), "Toggle");
pub const KEYBIND_CONTROLS_LIST_NAV: &UpDownKeybind = &UpDownKeybind::new("Navigate list");
pub const KEYBIND_CONTROLS_LIST_NAV_EXT: &UpDownExtendedKeybind =
    &UpDownExtendedKeybind::new("Navigate list");

pub const KEYBIND_MODAL_SUBMIT: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Enter, "Submit");
pub const KEYBIND_MODAL_SUBMITSELECT: &SimpleKeybind =
    &SimpleKeybind::new(KeyCode::Enter, "Select");
pub const KEYBIND_MODAL_CANCEL: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Esc, "Cancel");
pub const KEYBIND_MODAL_LEFTRIGHT_OPTION: &LeftRightKeybind =
    &LeftRightKeybind::new("Choose option");

pub const KEYBIND_SAVE: &SimpleKeybind =
    &SimpleKeybind::new_mod(KeyCode::Char('s'), KeyModifiers::CONTROL, "Save");
pub const KEYBIND_UNDO: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('u'), "Undo");
pub const KEYBIND_REDO: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('U'), "Redo");
pub const KEYBIND_QUIT: &SimpleKeybind = &SimpleKeybind::new(KeyCode::Char('q'), "Quit");
pub const KEYBIND_QUIT_ALT: &SimpleKeybind = &SimpleKeybind::new_hidden(KeyCode::Esc);

pub trait Keybind {
    fn is_match(&self, key: KeyEvent) -> bool;
    fn key_hint(&self) -> Cow<'static, str>;
    fn description(&self) -> Option<&Cow<'static, str>>;
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct KeyCombo(KeyCode, Option<KeyModifiers>);

impl KeyCombo {
    fn as_string(&self) -> Cow<'static, str> {
        // if shift is pressed, chars will already be the uppercase variant. this simplifies things.
        let mods_without_shift = self
            .1
            .map(|m| m.intersection(KeyModifiers::SHIFT.complement()));

        match (self.0, mods_without_shift) {
            (KeyCode::Up, Some(KeyModifiers::NONE) | None) => "↑".into(),
            (KeyCode::Down, Some(KeyModifiers::NONE) | None) => "↓".into(),
            (KeyCode::Left, Some(KeyModifiers::NONE) | None) => "←".into(),
            (KeyCode::Right, Some(KeyModifiers::NONE) | None) => "→".into(),
            (KeyCode::Enter, Some(KeyModifiers::NONE) | None) => "⏎".into(),
            (KeyCode::Tab, Some(KeyModifiers::NONE) | None) => "⭾".into(),
            (KeyCode::Esc, Some(KeyModifiers::NONE) | None) => "⎋".into(),

            (KeyCode::Char(c), Some(KeyModifiers::NONE) | None) => c.to_string().into(),
            (KeyCode::Char(c), Some(KeyModifiers::CONTROL)) => format!("^{c}").into(),

            _ => Cow::Owned("???".into()),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SimpleKeybind {
    key_combo: KeyCombo,
    description: Option<Cow<'static, str>>,
}

impl SimpleKeybind {
    pub const fn new(code: KeyCode, description: &'static str) -> Self {
        Self {
            key_combo: KeyCombo(code, None),
            description: Some(Cow::Borrowed(description)),
        }
    }

    pub const fn new_mod(
        code: KeyCode,
        modifiers: KeyModifiers,
        description: &'static str,
    ) -> Self {
        Self {
            key_combo: KeyCombo(code, Some(modifiers)),
            description: Some(Cow::Borrowed(description)),
        }
    }

    pub const fn new_hidden(code: KeyCode) -> Self {
        Self {
            key_combo: KeyCombo(code, None),
            description: None,
        }
    }
}

impl Keybind for SimpleKeybind {
    fn is_match(&self, key: KeyEvent) -> bool {
        self.key_combo.0 == key.code && self.key_combo.1.map(|x| x == key.modifiers).unwrap_or(true)
    }

    fn key_hint(&self) -> Cow<'static, str> {
        self.key_combo.as_string()
    }

    fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }
}

pub struct LeftRightKeybind {
    description: Option<Cow<'static, str>>,
}

impl LeftRightKeybind {
    pub const fn new(description: &'static str) -> Self {
        Self {
            description: Some(Cow::Borrowed(description)),
        }
    }

    pub fn get_match(&self, key: KeyEvent) -> Option<LeftRightKey> {
        match key.code {
            KeyCode::Left => Some(LeftRightKey::Left),
            KeyCode::Right => Some(LeftRightKey::Right),
            _ => None,
        }
    }
}

impl Keybind for LeftRightKeybind {
    fn is_match(&self, key: KeyEvent) -> bool {
        matches!(key.code, KeyCode::Left | KeyCode::Right)
    }

    fn key_hint(&self) -> Cow<'static, str> {
        "⇆".into()
    }

    fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }
}

pub enum LeftRightKey {
    Left,
    Right,
}

pub struct UpDownKeybind {
    description: Option<Cow<'static, str>>,
}

impl UpDownKeybind {
    pub const fn new(description: &'static str) -> Self {
        Self {
            description: Some(Cow::Borrowed(description)),
        }
    }

    pub fn get_match(&self, key: KeyEvent) -> Option<UpDownKey> {
        match key.code {
            KeyCode::Up => Some(UpDownKey::Up),
            KeyCode::Down => Some(UpDownKey::Down),
            _ => None,
        }
    }
}

impl Keybind for UpDownKeybind {
    fn is_match(&self, key: KeyEvent) -> bool {
        matches!(key.code, KeyCode::Up | KeyCode::Down)
    }

    fn key_hint(&self) -> Cow<'static, str> {
        "⇅".into()
    }

    fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }
}

pub enum UpDownKey {
    Up,
    Down,
}

pub struct UpDownExtendedKeybind {
    description: Option<Cow<'static, str>>,
}

impl UpDownExtendedKeybind {
    pub const fn new(description: &'static str) -> Self {
        Self {
            description: Some(Cow::Borrowed(description)),
        }
    }

    pub fn get_match(&self, key: KeyEvent) -> Option<UpDownExtendedKey> {
        match key.code {
            KeyCode::Up => Some(UpDownExtendedKey::Up),
            KeyCode::Down => Some(UpDownExtendedKey::Down),
            KeyCode::PageUp => Some(UpDownExtendedKey::PageUp),
            KeyCode::PageDown => Some(UpDownExtendedKey::PageDown),
            KeyCode::Home => Some(UpDownExtendedKey::Home),
            KeyCode::End => Some(UpDownExtendedKey::End),
            _ => None,
        }
    }
}

impl Keybind for UpDownExtendedKeybind {
    fn is_match(&self, key: KeyEvent) -> bool {
        matches!(
            key.code,
            KeyCode::Up
                | KeyCode::Down
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
        )
    }

    fn key_hint(&self) -> Cow<'static, str> {
        "⇅".into()
    }

    fn description(&self) -> Option<&Cow<'static, str>> {
        self.description.as_ref()
    }
}

pub enum UpDownExtendedKey {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

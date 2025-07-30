use std::fmt::Display;

use thiserror::Error;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::Settings;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("Failed to parse replay")]
    ParseError,

    #[error("Failed to read to buffer")]
    BufferError(#[from] std::io::Error),

    #[error("Failed to write replay")]
    WriteError,

    #[error("Failed to read replay")]
    Slc2Error(#[from] slc_oxide::replay::ReplayError),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[wasm_bindgen]
pub enum GameVersion {
    Any,
    Version2113,
    Version2206,
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "Any"),
            Self::Version2113 => write!(f, "2.113"),
            Self::Version2206 => write!(f, "2.206"),
        }
    }
}

#[derive(Clone)]
pub struct Replay {
    pub fps: f32,
    pub clicks: Vec<Click>,
    pub game_version: GameVersion,
    pub settings: Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[wasm_bindgen]
pub enum ClickType {
    Click,
    Release,
    Skip,
}

impl From<bool> for ClickType {
    fn from(value: bool) -> Self {
        if value {
            Self::Click
        } else {
            Self::Release
        }
    }
}

impl ClickType {
    #[must_use]
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }

    #[must_use]
    pub fn is_click(&self) -> bool {
        matches!(self, Self::Click)
    }

    #[must_use]
    pub fn is_release(&self) -> bool {
        matches!(self, Self::Release)
    }

    #[must_use]
    pub fn toggle(&self) -> Self {
        match self {
            Self::Click => Self::Release,
            Self::Release => Self::Skip,
            Self::Skip => Self::Click,
        }
    }
}

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct Click {
    pub frame: u32,
    pub p1: ClickType,
    pub p2: ClickType,
}

impl Click {
    #[must_use]
    pub fn from_hold(frame: u32, hold: bool, player_2: bool) -> Self {
        Self {
            frame,
            p1: if player_2 {
                ClickType::Skip
            } else {
                hold.into()
            },
            p2: if player_2 {
                hold.into()
            } else {
                ClickType::Skip
            },
        }
    }

    pub fn apply_hold<F, E>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(u32, bool, bool) -> Result<(), E>,
    {
        if !self.p1.is_skip() {
            f(self.frame, self.p1.is_click(), false)?;
        }

        if !self.p2.is_skip() {
            f(self.frame, self.p2.is_click(), true)?;
        }

        Ok(())
    }
}

impl Default for Replay {
    fn default() -> Self {
        Self::new(60.0, GameVersion::Version2113, Settings::default())
    }
}

impl Replay {
    #[must_use]
    pub fn new(fps: f32, game_version: GameVersion, settings: Settings) -> Self {
        Self {
            fps,
            clicks: vec![],
            game_version,
            settings,
        }
    }

    pub fn clear(&mut self) {
        self.clicks.clear();
        self.fps = 60.0;
    }
}

use std::fmt::Display;

use thiserror::Error;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("Failed to parse replay")]
    ParseError,

    #[error("Failed to read to buffer")]
    BufferError(#[from] std::io::Error),
    
    #[error("Failed to write replay")]
    WriteError,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[wasm_bindgen]
pub enum GameVersion {
    Any,
    Version2113,
    Version2206
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameVersion::Any => write!(f, "Any"),
            GameVersion::Version2113 => write!(f, "2.113"),
            GameVersion::Version2206 => write!(f, "2.206"),
        }
    }
}

#[derive(Clone)]
pub struct Replay {
    pub fps: f32,
    pub clicks: Vec<Click>,
    pub game_version: GameVersion
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[wasm_bindgen]
pub enum ClickType {
    Click,
    Release,
    Skip
}

impl From<bool> for ClickType {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Click,
            false => Self::Release
        }
    }
}

impl ClickType {
    pub fn is_skip(&self) -> bool {
        matches!(self, ClickType::Skip)
    }

    pub fn is_click(&self) -> bool {
        matches!(self, ClickType::Click)
    }

    pub fn is_release(&self) -> bool {
        matches!(self, ClickType::Release)
    }

    pub fn toggle(&self) -> Self {
        match self {
            ClickType::Click => ClickType::Release,
            ClickType::Release => ClickType::Click,
            ClickType::Skip => ClickType::Skip,
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
    pub fn from_hold(frame: u32, hold: bool, player_2: bool) -> Self {
        Self {
            frame,
            p1: if !player_2 { hold.into() } else { ClickType::Skip },
            p2: if player_2 { hold.into() } else { ClickType::Skip },
        }
    }

    pub fn apply_hold<F, E>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(u32, bool, bool) -> Result<(), E>
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
        Self::new(60.0, GameVersion::Version2113)
    }
}

impl Replay {
    pub fn new(fps: f32, game_version: GameVersion) -> Self {
        Self {
            fps,
            clicks: vec![],
            game_version
        }
    }

    pub fn clear(&mut self) {
        self.clicks.clear();
        self.fps = 60.0;
    }
}

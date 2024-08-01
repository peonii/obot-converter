use std::{fmt::Display, io::Cursor, path::Path};

use eyre::Result;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameVersion {
    Version2113,
    Version2206
}

impl Display for GameVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameVersion::Version2113 => write!(f, "2.113"),
            GameVersion::Version2206 => write!(f, "2.206"),
        }
    }
}

pub trait ReplayFormat {
    type ClickType;

    fn new(fps: f32) -> Self;
    fn from_data(data: &mut Cursor<Vec<u8>>) -> Result<Self>
    where
        Self: Sized;
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
 
    fn dump(&self) -> Result<Vec<u8>>;
    fn save(&self, path: impl AsRef<Path>) -> Result<()>;
    fn add_click(&mut self, click: Self::ClickType) -> ();

    fn from_universal(replay: Replay) -> Result<Self>
    where
        Self: Sized;
    fn to_universal(&self) -> Result<Replay>;
}

/// Made to act as an intermediate for converting between formats
/// Not serializable
#[derive(Clone)]
pub struct Replay {
    pub fps: f32,
    pub clicks: Vec<ReplayClick>,
    pub game_version: GameVersion
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[wasm_bindgen]
pub enum ReplayClickType {
    Click,
    Release,
    Skip,
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct ReplayClick {
    pub frame: i64,
    pub p1: ReplayClickType,
    pub p2: ReplayClickType,
}

impl Replay {
    pub fn new(fps: f32, game_version: GameVersion) -> Self {
        Self {
            fps,
            clicks: vec![],
            game_version
        }
    }
}

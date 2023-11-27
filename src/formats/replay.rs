use std::path::Path;

use eyre::Result;
use serde::ser;

pub trait ReplayFormat {
    type ClickType: ser::Serialize;

    fn new(fps: f32) -> Self;
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
    fn save(&self, path: impl AsRef<Path>) -> Result<()>;
    fn add_click(&mut self, click: Self::ClickType) -> ();

    fn from_universal(replay: Replay) -> Result<Self>
    where
        Self: Sized;
    fn to_universal(&self) -> Result<Replay>;
}

/// Made to act as an intermediate for converting between formats
/// Not serializable
pub struct Replay {
    pub fps: f32,
    pub clicks: Vec<ReplayClick>,
}

#[derive(Clone)]
pub enum ReplayClickType {
    Click,
    Release,
    Skip,
}

pub struct ReplayClick {
    pub frame: u32,
    pub p1: ReplayClickType,
    pub p2: ReplayClickType,
}

impl Replay {
    pub fn new(fps: f32) -> Self {
        Self {
            fps,
            clicks: vec![],
        }
    }
}

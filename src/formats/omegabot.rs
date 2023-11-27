use std::{fs::File, path::Path};

use dlhn::{Deserializer, Serializer};
use eyre::Result;
use serde::{Deserialize, Serialize};

use super::{
    replay::{Replay, ReplayClickType, ReplayFormat},
    tasbot::TasbotReplay,
};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default, Debug)]
pub enum OmegabotClickType {
    #[default]
    None,

    Player1Down,
    Player1Up,
    Player2Down,
    Player2Up,

    FpsChange(f32),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct OmegabotClick {
    pub frame: u32,
    pub click_type: OmegabotClickType,
}

#[derive(Serialize, Deserialize)]
pub struct OmegabotReplay {
    pub initial_fps: f32,
    current_fps: f32,
    pub clicks: Vec<OmegabotClick>,
    current: usize,
}

impl ReplayFormat for OmegabotReplay {
    type ClickType = OmegabotClick;

    fn new(fps: f32) -> Self {
        Self {
            initial_fps: fps,
            current_fps: fps,
            clicks: Vec::new(),
            current: 0,
        }
    }

    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut deserializer = Deserializer::new(&mut file);
        Self::deserialize(&mut deserializer)
            .map_err(|e| e.into())
            .map(|mut replay| {
                replay.current_fps = replay.initial_fps;
                replay.current = 0;
                replay
            })
    }

    fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = File::create(path)?;
        let mut serializer = Serializer::new(&mut file);
        self.serialize(&mut serializer).map_err(|e| e.into())
    }

    fn add_click(&mut self, click: OmegabotClick) {
        if let OmegabotClickType::FpsChange(fps) = click.click_type {
            self.current_fps = fps;
        }

        self.clicks.push(click);
        self.current = self.clicks.len();
    }

    fn from_universal(replay: super::replay::Replay) -> Result<Self> {
        let mut obot_replay = Self::new(replay.fps);
        for click in replay.clicks.iter() {
            obot_replay.add_click(OmegabotClick {
                frame: click.frame,
                click_type: match click.p1 {
                    ReplayClickType::Click => OmegabotClickType::Player1Down,
                    ReplayClickType::Release => OmegabotClickType::Player1Up,
                    ReplayClickType::Skip => OmegabotClickType::None,
                },
            });

            obot_replay.add_click(OmegabotClick {
                frame: click.frame,
                click_type: match click.p2 {
                    ReplayClickType::Click => OmegabotClickType::Player2Down,
                    ReplayClickType::Release => OmegabotClickType::Player2Up,
                    ReplayClickType::Skip => OmegabotClickType::None,
                },
            });
        }

        Ok(obot_replay)
    }

    fn to_universal(&self) -> Result<super::replay::Replay> {
        let mut replay = Replay::new(self.initial_fps);

        for click in self.clicks.iter() {
            let mut click_type = ReplayClickType::Skip;
            let mut p2 = false;

            match click.click_type {
                OmegabotClickType::Player1Up => {
                    click_type = ReplayClickType::Release;
                }
                OmegabotClickType::Player2Up => {
                    p2 = true;
                    click_type = ReplayClickType::Release;
                }
                OmegabotClickType::Player1Down => {
                    click_type = ReplayClickType::Click;
                }
                OmegabotClickType::Player2Down => {
                    p2 = true;
                    click_type = ReplayClickType::Click;
                }
                _ => {}
            }

            replay.clicks.push(super::replay::ReplayClick {
                frame: click.frame,
                p1: if !p2 {
                    click_type.clone()
                } else {
                    ReplayClickType::Skip
                },
                p2: if p2 {
                    click_type.clone()
                } else {
                    ReplayClickType::Skip
                },
            })
        }

        Ok(replay)
    }
}

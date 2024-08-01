use std::{fs::File, io::Cursor, path::Path};

use dlhn::{Deserializer, Serializer};
use eyre::Result;
use serde::{Deserialize, Serialize};

use super::replay::{GameVersion, Replay, ReplayClickType, ReplayFormat};

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

impl OmegabotClickType {
    pub fn is_player1(&self) -> bool {
        matches!(self, OmegabotClickType::Player1Down | OmegabotClickType::Player1Up)
    }

    pub fn is_player2(&self) -> bool {
        matches!(self, OmegabotClickType::Player2Down | OmegabotClickType::Player2Up)
    }
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
    
    fn from_data(data: &mut Cursor<Vec<u8>>) -> Result<Self>
        where
            Self: Sized {
        
        let mut deserializer = Deserializer::new(data);
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

    fn dump(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut serializer = Serializer::new(&mut data);
        self.serialize(&mut serializer)?;
        Ok(data)
    }

    fn add_click(&mut self, click: OmegabotClick) {
        if let OmegabotClickType::FpsChange(fps) = click.click_type {
            self.current_fps = fps;
        }

        self.clicks.push(click);
        self.current = self.clicks.len();
    }

    fn from_universal(replay: super::replay::Replay) -> Result<Self> {
        if replay.game_version != GameVersion::Version2113 {
            return Err(eyre::eyre!("Unsupported game version: {:?}", replay.game_version));
        }

        let mut obot_replay = Self::new(replay.fps);
        for click in replay.clicks.iter() {
            match click.p1 {
                ReplayClickType::Click => {
                    obot_replay.add_click(OmegabotClick {
                        frame: click.frame as u32,
                        click_type: OmegabotClickType::Player1Down,
                    });
                }
                ReplayClickType::Release => {
                    obot_replay.add_click(OmegabotClick {
                        frame: click.frame as u32,
                        click_type: OmegabotClickType::Player1Up,
                    });
                }
                _ => {}
            }
            
            match click.p2 {
                ReplayClickType::Click => {
                    obot_replay.add_click(OmegabotClick {
                        frame: click.frame as u32,
                        click_type: OmegabotClickType::Player2Down,
                    });
                }
                ReplayClickType::Release => {
                    obot_replay.add_click(OmegabotClick {
                        frame: click.frame as u32,
                        click_type: OmegabotClickType::Player2Up,
                    });
                }
                _ => {}
            }
        }

        Ok(obot_replay)
    }

    fn to_universal(&self) -> Result<super::replay::Replay> {
        let mut replay = Replay::new(self.initial_fps, GameVersion::Version2113);

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
                _ => {
                    continue;
                }
            }

            // let previous_click = replay.clicks.last_mut();

            // if let Some(p) = previous_click {
            //     if p.frame == click.frame {
            //         if p.p2 == ReplayClickType::Skip && p2 {
            //             p.p2 = click_type;
            //             continue;
            //         } else if p.p1 == ReplayClickType::Skip && !p2 {
            //             p.p1 = click_type;
            //             continue;
            //         }
            //     }
            // }

            replay.clicks.push(super::replay::ReplayClick {
                frame: click.frame as i64,
                p1: if !p2 {
                    click_type
                } else {
                    ReplayClickType::Skip
                },
                p2: if p2 {
                    click_type
                } else {
                    ReplayClickType::Skip
                },
            })
        }

        Ok(replay)
    }
}

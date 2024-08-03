use serde::{Deserialize, Serialize};

use super::replay::{GameVersion, ReplayClickType, ReplayFormat};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Location {
    Frame(u32),
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Location::Frame(x), Location::Frame(y)) => x == y,
            _ => false,
        }
    }
}

impl Eq for Location {}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Location::Frame(x), Location::Frame(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("Cannot compare locations of different types")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ClickType {
    None,
    FpsChange(f32),
    Player1Down,
    Player1Up,
    Player2Down,
    Player2Up,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Click {
    pub location: Location,
    pub click_type: ClickType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmegaBot2ReplayType {
    Frame,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct OmegaBot2Replay {
    pub(crate) initial_fps: f32,
    pub(crate) current_fps: f32,
    pub(crate) replay_type: OmegaBot2ReplayType,
    pub(crate) current_click: usize,
    pub(crate) clicks: Vec<Click>,
}

impl ReplayFormat for OmegaBot2Replay {
    type ClickType = Click;

    fn new(fps: f32) -> Self {
        Self {
            initial_fps: fps,
            current_fps: fps,
            replay_type: OmegaBot2ReplayType::Frame,
            current_click: 0,
            clicks: Vec::new(),
        }
    }

    fn add_click(&mut self, click: Self::ClickType) -> () {
        self.clicks.push(click);
    }

    fn from_data(data: &mut std::io::Cursor<Vec<u8>>) -> eyre::Result<Self>
        where
            Self: Sized {
        let deserialized = bincode::deserialize_from(data)?;

        Ok(deserialized)
    }

    fn dump(&self) -> eyre::Result<Vec<u8>> {
        let data = bincode::serialize(&self)?;

        Ok(data)
    }

    fn from_universal(replay: super::replay::Replay) -> eyre::Result<Self>
        where
            Self: Sized {
        if replay.game_version != GameVersion::Version2113 {
            return Err(eyre::eyre!("Unsupported game version: {:?}", replay.game_version));
        }

        let mut obot_replay = Self::new(replay.fps);
        for click in replay.clicks.iter() {
            match click.p1 {
                ReplayClickType::Click => {
                    obot_replay.add_click(Click {
                        location: Location::Frame(click.frame as u32),
                        click_type: ClickType::Player1Down,
                    });
                }
                ReplayClickType::Release => {
                    obot_replay.add_click(Click {
                        location: Location::Frame(click.frame as u32),
                        click_type: ClickType::Player1Up,
                    });
                }
                _ => {}
            }
            
            match click.p2 {
                ReplayClickType::Click => {
                    obot_replay.add_click(Click {
                        location: Location::Frame(click.frame as u32),
                        click_type: ClickType::Player2Down,
                    });
                }
                ReplayClickType::Release => {
                    obot_replay.add_click(Click {
                        location: Location::Frame(click.frame as u32),
                        click_type: ClickType::Player2Up,
                    });
                }
                _ => {}
            }
        }

        Ok(obot_replay)
    }

    fn to_universal(&self) -> eyre::Result<super::replay::Replay> {
        let mut replay = super::replay::Replay::new(self.initial_fps, super::replay::GameVersion::Version2113);

        for click in &self.clicks {
            let frame = match click.location {
                Location::Frame(x) => x,
            };

            match click.click_type {
                ClickType::Player1Down => {
                    replay.clicks.push(
                        super::replay::ReplayClick {
                            frame: frame as i64,
                            p1: ReplayClickType::Click,
                            p2: ReplayClickType::Skip,
                        }
                    )
                }
                ClickType::Player1Up => {
                    replay.clicks.push(
                        super::replay::ReplayClick {
                            frame: frame as i64,
                            p1: ReplayClickType::Release,
                            p2: ReplayClickType::Skip,
                        }
                    )
                }
                ClickType::Player2Down => {
                    replay.clicks.push(
                        super::replay::ReplayClick {
                            frame: frame as i64,
                            p1: ReplayClickType::Skip,
                            p2: ReplayClickType::Click,
                        }
                    )
                }
                ClickType::Player2Up => {
                    replay.clicks.push(
                        super::replay::ReplayClick {
                            frame: frame as i64,
                            p1: ReplayClickType::Skip,
                            p2: ReplayClickType::Release,
                        }
                    )
                }
                _ => {}
            }
        }

        Ok(replay)
    }

    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self>
        where
            Self: Sized {
        let data = std::fs::read(path)?;

        let mut cursor = std::io::Cursor::new(data);

        Self::from_data(&mut cursor)
    }

    fn save(&self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
        let data = self.dump()?;

        std::fs::write(path, data)?;

        Ok(())
    }
}
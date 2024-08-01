use std::fs::File;

use serde::{Deserialize, Serialize};

use super::replay::{GameVersion, Replay, ReplayClick, ReplayFormat};

#[derive(Serialize, Deserialize)]
pub struct TasbotReplay {
    pub fps: f32,
    #[serde(rename = "macro")]
    pub clicks: Vec<TasbotClick>,
}

#[derive(Serialize, Deserialize)]
pub struct TasbotClick {
    pub frame: i32,
    pub player_1: TasbotAction,
    pub player_2: TasbotAction,
}

#[derive(Serialize, Deserialize)]
pub struct TasbotAction {
    // 0 = nothing, 1 = down, 2 = up
    pub click: u32,
    pub x_position: f64,
}

impl ReplayFormat for TasbotReplay {
    type ClickType = TasbotClick;

    fn new(fps: f32) -> Self {
        Self {
            fps,
            clicks: vec![],
        }
    }

    fn from_data(data: &mut std::io::Cursor<Vec<u8>>) -> eyre::Result<Self>
        where
            Self: Sized {
        let deserialized = serde_json::from_reader::<std::io::Cursor<Vec<u8>>, Self>(data.clone())?;
        Ok(deserialized)
    }

    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self> {
        let file = File::open(path)?;
        let deserialized = serde_json::from_reader::<File, Self>(file)?;
        Ok(deserialized)
    }

    fn dump(&self) -> eyre::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn save(&self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    fn add_click(&mut self, click: Self::ClickType) -> () {
        self.clicks.push(click);
    }

    fn from_universal(replay: Replay) -> eyre::Result<Self> {
        if replay.game_version != GameVersion::Version2113 {
            return Err(eyre::eyre!("Unsupported game version: {:?}", replay.game_version));
        }

        let mut tbot_replay = Self::new(replay.fps);

        for click in replay.clicks {
            tbot_replay.add_click(TasbotClick {
                frame: if click.frame != 0 { click.frame as i32 - 1 } else { 0 },
                player_1: TasbotAction {
                    click: match click.p1 {
                        super::replay::ReplayClickType::Click => 1,
                        super::replay::ReplayClickType::Release => 2,
                        super::replay::ReplayClickType::Skip => 0,
                    },
                    x_position: 0.0,
                },
                player_2: TasbotAction {
                    click: match click.p2 {
                        super::replay::ReplayClickType::Click => 1,
                        super::replay::ReplayClickType::Release => 2,
                        super::replay::ReplayClickType::Skip => 0,
                    },
                    x_position: 0.0,
                },
            })
        }

        Ok(tbot_replay)
    }

    fn to_universal(&self) -> eyre::Result<Replay> {
        let mut replay = Replay::new(self.fps, GameVersion::Version2113);

        for click in self.clicks.iter() {
            replay.clicks.push(ReplayClick {
                frame: if click.frame != 0 { click.frame as i64 + 1 } else { 0 },
                p1: match click.player_1.click {
                    0 => super::replay::ReplayClickType::Skip,
                    1 => super::replay::ReplayClickType::Click,
                    2 => super::replay::ReplayClickType::Release,
                    _ => panic!("Invalid action"),
                },
                p2: match click.player_2.click {
                    0 => super::replay::ReplayClickType::Skip,
                    1 => super::replay::ReplayClickType::Click,
                    2 => super::replay::ReplayClickType::Release,
                    _ => panic!("Invalid action"),
                },
            })
        }

        Ok(replay)
    }
}

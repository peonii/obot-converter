use std::fs::File;

use serde::{Deserialize, Serialize};

use super::replay::{GameVersion, Replay, ReplayFormat};

#[derive(Serialize, Deserialize)]
pub struct MHRReplay {
    #[serde(rename = "_")]
    pub tag: String,
    pub events: Vec<MHRClick>,
    pub meta: MHRMeta,
}

#[derive(Serialize, Deserialize)]
pub struct MHRClick {
    pub frame: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct MHRMeta {
    pub fps: f32,
}

impl ReplayFormat for MHRReplay {
    type ClickType = MHRClick;

    fn new(fps: f32) -> Self {
        Self {
            tag: "OmegaBot converter".to_owned(),
            events: vec![],
            meta: MHRMeta { fps },
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
        self.events.push(MHRClick {
            frame: click.frame,
            a: Some(0.0),
            r: Some(0.0),
            x: Some(0.0),
            y: Some(0.0),
            down: click.down,
            p2: click.p2,
        })
    }

    fn from_universal(replay: super::replay::Replay) -> eyre::Result<Self> {
        if replay.game_version != GameVersion::Version2113 {
            return Err(eyre::eyre!("Unsupported game version: {:?}", replay.game_version));
        }

        let mut mhr_replay = MHRReplay::new(replay.fps);

        for click in replay.clicks.iter() {
            if click.p1 != super::replay::ReplayClickType::Skip {
                mhr_replay.add_click(MHRClick {
                    frame: click.frame as u32,
                    a: Some(0.0),
                    r: Some(0.0),
                    x: Some(0.0),
                    y: Some(0.0),
                    down: match click.p1 {
                        super::replay::ReplayClickType::Skip => None,
                        super::replay::ReplayClickType::Click => Some(true),
                        super::replay::ReplayClickType::Release => Some(false),
                    },
                    p2: None
                });
            }

            if click.p2 != super::replay::ReplayClickType::Skip {
                mhr_replay.add_click(MHRClick {
                    frame: click.frame as u32,
                    a: Some(0.0),
                    r: Some(0.0),
                    x: Some(0.0),
                    y: Some(0.0),
                    down: match click.p2 {
                        super::replay::ReplayClickType::Skip => None,
                        super::replay::ReplayClickType::Click => Some(true),
                        super::replay::ReplayClickType::Release => Some(false),
                    },
                    p2: Some(true)
                });
            }
        }

        Ok(mhr_replay)
    }

    fn to_universal(&self) -> eyre::Result<super::replay::Replay> {
        let mut replay = Replay::new(self.meta.fps, GameVersion::Version2113);

        for click in self.events.iter() {
            if click.p2.is_some_and(|p2| p2) {
                replay.clicks.push(super::replay::ReplayClick {
                    frame: click.frame as i64,
                    p1: super::replay::ReplayClickType::Skip,
                    p2: match click.down {
                        Some(true) => super::replay::ReplayClickType::Click,
                        Some(false) => super::replay::ReplayClickType::Release,
                        None => super::replay::ReplayClickType::Skip,
                    },
                });
            } else {
                replay.clicks.push(super::replay::ReplayClick {
                    frame: click.frame as i64,
                    p1: match click.down {
                        Some(true) => super::replay::ReplayClickType::Click,
                        Some(false) => super::replay::ReplayClickType::Release,
                        None => super::replay::ReplayClickType::Skip,
                    },
                    p2: super::replay::ReplayClickType::Skip,
                });
            }
        }

        Ok(replay)
    }
}

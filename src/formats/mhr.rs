use std::fs::File;

use serde::{Deserialize, Serialize};

use super::replay::{Replay, ReplayFormat};

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

    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self> {
        let file = File::open(path)?;
        let deserialized = simd_json::from_reader::<File, Self>(file)?;
        Ok(deserialized)
    }

    fn save(&self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
        let file = File::create(path)?;
        simd_json::to_writer(file, self)?;
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
        let mut mhr_replay = MHRReplay::new(replay.fps);

        for click in replay.clicks.iter() {
            mhr_replay.add_click(MHRClick {
                frame: click.frame,
                a: Some(0.0),
                r: Some(0.0),
                x: Some(0.0),
                y: Some(0.0),
                down: match click.p1 {
                    super::replay::ReplayClickType::Skip => None,
                    super::replay::ReplayClickType::Click => Some(true),
                    super::replay::ReplayClickType::Release => Some(false),
                },
                p2: None,
            });

            mhr_replay.add_click(MHRClick {
                frame: click.frame,
                a: Some(0.0),
                r: Some(0.0),
                x: Some(0.0),
                y: Some(0.0),
                down: match click.p2 {
                    super::replay::ReplayClickType::Skip => None,
                    super::replay::ReplayClickType::Click => Some(true),
                    super::replay::ReplayClickType::Release => Some(false),
                },
                p2: Some(true),
            });
        }

        Ok(mhr_replay)
    }

    fn to_universal(&self) -> eyre::Result<super::replay::Replay> {
        let mut replay = Replay::new(self.meta.fps);

        for click in self.events.iter() {
            if let Some(down) = click.down {
                replay.clicks.push(super::replay::ReplayClick {
                    frame: click.frame,
                    p1: match down {
                        true => super::replay::ReplayClickType::Click,
                        false => super::replay::ReplayClickType::Release,
                    },
                    p2: super::replay::ReplayClickType::Skip,
                });
            }

            if let Some(p2) = click.p2 {
                replay.clicks.push(super::replay::ReplayClick {
                    frame: click.frame,
                    p1: super::replay::ReplayClickType::Skip,
                    p2: match p2 {
                        true => super::replay::ReplayClickType::Click,
                        false => super::replay::ReplayClickType::Release,
                    },
                });
            }
        }

        Ok(replay)
    }
}

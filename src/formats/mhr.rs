use std::io::{BufReader, BufWriter, Read, Seek, Write};

use serde::{Deserialize, Serialize};

use super::replay::{Click, GameVersion, Replay, ReplayError};

#[derive(Serialize, Deserialize)]
struct MHRReplay {
    #[serde(rename = "_")]
    pub tag: String,
    pub events: Vec<MHRClick>,
    pub meta: MHRMeta,
}

#[derive(Serialize, Deserialize)]
struct MHRClick {
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

impl From<MHRClick> for Click {
    fn from(value: MHRClick) -> Self {
        Click::from_hold(value.frame, value.down.unwrap_or(false), value.p2.unwrap_or(false))
    }
}

#[derive(Serialize, Deserialize)]
struct MHRMeta {
    pub fps: f32,
}


impl Replay {
    pub fn parse_mhr_json(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let reader = BufReader::new(reader);

        let replay: MHRReplay = serde_json::from_reader(reader)
            .map_err(|_| ReplayError::ParseError)?;

        self.fps = replay.meta.fps;
        self.clicks = replay.events.into_iter().map(|click| click.into()).collect();
        self.game_version = GameVersion::Version2113;

        Ok(())
    }

    pub fn write_mhr_json(&self, writer: impl Write + Seek) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        let mut clicks = Vec::new();
        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                clicks.push(MHRClick {
                    frame,
                    down: Some(hold),
                    p2: if p2 { Some(p2) } else { None },
                    a: None,
                    x: None,
                    r: None,
                    y: None
                });

                Ok::<(), ReplayError>(())
            })
        })?;

        let replay = MHRReplay {
            tag: "converter by nat :3".to_owned(),
            events: clicks,
            meta: MHRMeta {
                fps: self.fps
            }
        };

        serde_json::to_writer(&mut writer, &replay)
            .map_err(|_| ReplayError::WriteError)?;

        Ok(())
    }
}
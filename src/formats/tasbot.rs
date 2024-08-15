use std::io::{BufReader, BufWriter, Read, Seek, Write};

use serde::{Deserialize, Serialize};

use crate::formats::replay::ClickType;

use super::replay::{Click, GameVersion, Replay, ReplayError};

#[derive(Serialize, Deserialize)]
struct TasbotReplay {
    pub fps: f32,
    #[serde(rename = "macro")]
    pub clicks: Vec<TasbotClick>,
}

#[derive(Serialize, Deserialize)]
struct TasbotClick {
    pub frame: u32,
    pub player_1: TasbotAction,
    pub player_2: TasbotAction,
}

#[derive(Serialize, Deserialize)]
struct TasbotAction {
    // 0 = nothing, 1 = down, 2 = up
    pub click: u32,
    pub x_position: f64,
}

impl Replay {
    pub fn parse_tasbot(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let reader = BufReader::new(reader);

        let replay: TasbotReplay =
            simd_json::from_reader(reader).map_err(|_| ReplayError::ParseError)?;

        let offset = self.settings.auto_offset as u32;

        self.fps = replay.fps;
        self.clicks = replay
            .clicks
            .into_iter()
            .map(|click| Click {
                frame: click.frame + offset,
                p1: match click.player_1.click {
                    1 => ClickType::Click,
                    2 => ClickType::Release,
                    _ => ClickType::Skip,
                },
                p2: match click.player_2.click {
                    1 => ClickType::Click,
                    2 => ClickType::Release,
                    _ => ClickType::Skip,
                },
            })
            .collect();
        self.game_version = GameVersion::Version2113;

        Ok(())
    }

    pub fn write_tasbot(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let writer = BufWriter::new(writer);

        let offset = self.settings.auto_offset as u32;

        let clicks = self
            .clicks
            .iter()
            .map(|click| TasbotClick {
                frame: click.frame - offset,
                player_1: TasbotAction {
                    x_position: 0.0,
                    click: match click.p1 {
                        ClickType::Click => 1,
                        ClickType::Release => 2,
                        ClickType::Skip => 0,
                    },
                },
                player_2: TasbotAction {
                    x_position: 0.0,
                    click: match click.p2 {
                        ClickType::Click => 1,
                        ClickType::Release => 2,
                        ClickType::Skip => 0,
                    },
                },
            })
            .collect::<Vec<TasbotClick>>();

        let replay = TasbotReplay {
            fps: self.fps,
            clicks,
        };

        if self.settings.beautified_json {
            serde_json::to_writer_pretty(writer, &replay).map_err(|_| ReplayError::WriteError)?;
        } else {
            serde_json::to_writer(writer, &replay).map_err(|_| ReplayError::WriteError)?;
        }

        Ok(())
    }
}

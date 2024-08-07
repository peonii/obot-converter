use std::io::{BufReader, BufWriter, Read, Seek, Write};

use serde::{Deserialize, Serialize};

use crate::formats::replay::{Click, ClickType};

use super::replay::{Replay, ReplayError};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default, Debug)]
enum OmegabotClickType {
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

impl From<OmegabotClickType> for ClickType {
    fn from(value: OmegabotClickType) -> ClickType {
        match value {
            OmegabotClickType::Player1Down => ClickType::Click,
            OmegabotClickType::Player1Up => ClickType::Release,
            OmegabotClickType::Player2Down => ClickType::Click,
            OmegabotClickType::Player2Up => ClickType::Release,
            _ => ClickType::Skip
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct OmegabotClick {
    pub frame: u32,
    pub click_type: OmegabotClickType,
}

impl From<OmegabotClick> for Click {
    fn from(value: OmegabotClick) -> Click {
        Click {
            frame: value.frame,
            p1: if value.click_type.is_player1() { value.click_type.into() } else { ClickType::Skip },
            p2: if value.click_type.is_player2() { value.click_type.into() } else { ClickType::Skip },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct OmegabotReplay {
    pub initial_fps: f32,
    current_fps: f32,
    pub clicks: Vec<OmegabotClick>,
    current: usize,
}


impl Replay {
    pub fn parse_obot3(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut reader = BufReader::new(reader);

        let mut deserializer = dlhn::Deserializer::new(&mut reader);
        let replay = OmegabotReplay::deserialize(&mut deserializer)
            .map_err(|_| ReplayError::ParseError)?;

        self.fps = replay.initial_fps;
        self.clicks = replay.clicks.into_iter().map(|click| click.into()).collect();

        Ok(())
    }

    pub fn write_obot3(&self, writer: impl Write + Seek) -> Result<(), ReplayError> {
        let writer = BufWriter::new(writer);

        let mut serializer = dlhn::Serializer::new(writer);

        let mut clicks: Vec<OmegabotClick> = vec![];
        for click in self.clicks.iter() {
            click.apply_hold(|frame, hold, p2| {
                let click_type;

                if hold && p2 {
                    click_type = OmegabotClickType::Player2Down
                } else if hold && !p2 {
                    click_type = OmegabotClickType::Player1Down
                } else if !hold && p2 {
                    click_type = OmegabotClickType::Player2Up
                } else {
                    click_type = OmegabotClickType::Player1Up
                }

                clicks.push(OmegabotClick {
                    frame,
                    click_type
                });

                Ok::<(), ReplayError>(())
            })?;
        }

        let replay = OmegabotReplay {
            initial_fps: self.fps,
            current_fps: self.fps,
            current: 0,
            clicks
        };

        replay.serialize(&mut serializer)
            .map_err(|_| ReplayError::WriteError)?;

        Ok(())
    }
}

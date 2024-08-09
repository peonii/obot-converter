use std::io::{BufReader, BufWriter, Read, Seek, Write};

use serde::{Deserialize, Serialize};

use super::replay::{Click, GameVersion, Replay, ReplayError};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum OmegaBot2Location {
    XPos(u32),
    Frame(u32),
}

impl PartialEq for OmegaBot2Location {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OmegaBot2Location::Frame(x), OmegaBot2Location::Frame(y)) => x == y,
            _ => false,
        }
    }
}

impl Eq for OmegaBot2Location {}

impl PartialOrd for OmegaBot2Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (OmegaBot2Location::Frame(x), OmegaBot2Location::Frame(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl Ord for OmegaBot2Location {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("Cannot compare locations of different types")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
enum OmegaBot2ClickType {
    None,
    FpsChange(f32),
    Player1Down,
    Player1Up,
    Player2Down,
    Player2Up,
}

impl OmegaBot2ClickType {
    pub fn is_player_2(&self) -> bool {
        matches!(self, Self::Player2Up | Self::Player2Down)
    }

    pub fn is_down(&self) -> bool {
        matches!(self, Self::Player1Down | Self::Player2Down)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct OmegaBot2Click {
    pub location: OmegaBot2Location,
    pub click_type: OmegaBot2ClickType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
enum OmegaBot2ReplayType {
    XPos,
    Frame,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct OmegaBot2Replay {
    pub(crate) initial_fps: f32,
    pub(crate) current_fps: f32,
    pub(crate) replay_type: OmegaBot2ReplayType,
    pub(crate) current_click: usize,
    pub(crate) clicks: Vec<OmegaBot2Click>,
}

impl TryFrom<&OmegaBot2Click> for Click {
    type Error = ReplayError;

    fn try_from(value: &OmegaBot2Click) -> Result<Self, Self::Error> {
        let frame = match value.location {
            OmegaBot2Location::Frame(f) => Ok(f + 1),
            OmegaBot2Location::XPos(_) => Err(ReplayError::ParseError)
        }?;

        let hold = value.click_type.is_down();
        let p2 = value.click_type.is_player_2();

        Ok(Click::from_hold(frame, hold, p2))
    }
}

impl Replay {
    pub fn parse_obot2(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let reader = BufReader::new(reader);

        let replay: OmegaBot2Replay = bincode::deserialize_from(reader)
            .map_err(|_| ReplayError::ParseError)?;

        self.fps = replay.initial_fps;

        self.clicks.reserve(replay.clicks.len());
        self.game_version = GameVersion::Version2113;

        replay.clicks.iter().try_for_each(|click| {
            self.clicks.push(click.try_into()?);

            Ok::<(), ReplayError>(())
        })?;

        Ok(())
    }

    pub fn write_obot2(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        let mut clicks = Vec::new();
        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                let click_type;

                if hold && p2 {
                    click_type = OmegaBot2ClickType::Player2Down
                } else if hold && !p2 {
                    click_type = OmegaBot2ClickType::Player1Down
                } else if !hold && p2 {
                    click_type = OmegaBot2ClickType::Player2Up
                } else {
                    click_type = OmegaBot2ClickType::Player1Up
                }

                clicks.push(OmegaBot2Click {
                    location: OmegaBot2Location::Frame(frame - 1),
                    click_type
                });

                Ok::<(), ReplayError>(())
            })
        })?;

        let replay = OmegaBot2Replay {
            clicks,
            initial_fps: self.fps,
            current_fps: self.fps,
            current_click: 0,
            replay_type: OmegaBot2ReplayType::Frame
        };

        bincode::serialize_into(&mut writer, &replay)
            .map_err(|_| ReplayError::WriteError)?;

        Ok(())
    }
}
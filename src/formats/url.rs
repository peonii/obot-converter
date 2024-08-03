use std::{fs::File, io::{Cursor, Read, Seek}};

use super::replay::{self, ReplayFormat};


pub struct URLReplay {
    pub clicks: Vec<URLClick>,
    pub fps: f32
}

pub struct URLClick {
    pub hold: bool,
    pub player_2: bool,
    pub frame: u32
}

pub enum URLReplayType {
    XPos,
    Frames,
    Both
}

impl From<u8> for URLReplayType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::XPos,
            1 => Self::Frames,
            2 => Self::Both,
            _ => panic!("Invalid URLReplayType")
        }
    }
}

impl Into<u8> for URLReplayType {
    fn into(self) -> u8 {
        match self {
            Self::XPos => 0,
            Self::Frames => 1,
            Self::Both => 2
        }
    }
}

impl URLClick {
    pub fn from_binary(data: &mut Cursor<Vec<u8>>, replay_type: &URLReplayType) -> eyre::Result<Self> {
        let mut state = [0u8; 1];
        data.read_exact(&mut state)?;
        let state = state[0];

        let hold = state & 1 == 1;
        let player_2 = state >> 1 == 1;

        let frame = match replay_type {
            URLReplayType::XPos => return Err(eyre::eyre!("XPos replay type not supported")),
            URLReplayType::Frames => {
                let mut frame = [0u8; 4];
                data.read_exact(&mut frame)?;
                u32::from_le_bytes(frame)
            },
            URLReplayType::Both => {
                let mut frame = [0u8; 4];
                // Skip xpos
                data.seek(std::io::SeekFrom::Current(4))?;
                data.read_exact(&mut frame)?;
                u32::from_le_bytes(frame)
            }
        };

        Ok(Self {
            hold,
            player_2,
            frame
        })
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = vec![];

        let state = if self.hold { 1 } else { 0 } | if self.player_2 { 2 } else { 0 };
        data.push(state);

        data.extend_from_slice(&self.frame.to_le_bytes());

        data
    }
}

impl ReplayFormat for URLReplay {
    type ClickType = URLClick;

    fn new(fps: f32) -> Self {
        Self {
            clicks: vec![],
            fps
        }
    }

    fn add_click(&mut self, click: Self::ClickType) -> () {
        self.clicks.push(click);
    }

    fn from_data(data: &mut Cursor<Vec<u8>>) -> eyre::Result<Self>
        where
            Self: Sized {
        let fps = {
            let mut fps = [0u8; 4];
            data.read_exact(&mut fps)?;
            f32::from_le_bytes(fps)
        };

        let replay_type = {
            let mut replay_type = [0u8; 1];
            data.read_exact(&mut replay_type)?;
            URLReplayType::from(replay_type[0])
        };

        let mut replay = Self::new(fps);

        loop {
            match URLClick::from_binary(data, &replay_type) {
                Ok(click) => replay.add_click(click),
                Err(_) => break
            }
        }

        Ok(replay)
    }

    fn dump(&self) -> eyre::Result<Vec<u8>> {
        let mut data = vec![];

        data.extend_from_slice(&self.fps.to_le_bytes());

        data.push(URLReplayType::Frames.into());

        for click in &self.clicks {
            data.extend_from_slice(&click.to_binary());
        }

        Ok(data)    
    }

    fn from_universal(replay: replay::Replay) -> eyre::Result<Self>
        where
            Self: Sized {
        let mut url = Self::new(replay.fps);

        for click in replay.clicks {
            if click.p2 != replay::ReplayClickType::Skip {
                url.add_click(URLClick {
                    hold: click.p2 == replay::ReplayClickType::Click,
                    player_2: true,
                    frame: click.frame as u32
                });
            }

            if click.p1 != replay::ReplayClickType::Skip {
                url.add_click(URLClick {
                    hold: click.p1 == replay::ReplayClickType::Click,
                    player_2: false,
                    frame: click.frame as u32
                });
            }
        }

        Ok(url)
    }

    fn to_universal(&self) -> eyre::Result<replay::Replay> {
        let mut replay = replay::Replay::new(self.fps, replay::GameVersion::Version2113);

        for click in &self.clicks {
            let click_type = if click.hold {
                replay::ReplayClickType::Click
            } else {
                replay::ReplayClickType::Release
            };

            replay.clicks.push(replay::ReplayClick {
                frame: click.frame as i64,
                p1: if click.player_2 { replay::ReplayClickType::Skip } else { click_type },
                p2: if click.player_2 { click_type } else { replay::ReplayClickType::Skip }
            });
        }

        Ok(replay)
    }

    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self>
        where
            Self: Sized {
        let data = std::fs::read(path)?;

        let mut cursor = Cursor::new(data);
        Self::from_data(&mut cursor)
    }

    fn save(&self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
        let data = self.dump()?;
        std::fs::write(path, data)?;
        Ok(())
    }
}


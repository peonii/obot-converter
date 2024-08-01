use std::io::{Read, Write};

use super::replay::{GameVersion, ReplayClickType, ReplayFormat};


pub struct ZBotReplay {
    pub fps: f32,
    pub clicks: Vec<ZBotClick>,
}

pub struct ZBotClick {
    pub frame: i32,
    pub hold: bool,
    pub player_1: bool
}

impl ZBotClick {
    pub fn from_binary(data: &mut std::io::Cursor<Vec<u8>>) -> eyre::Result<Self> {
        let mut frame = [0; 4];
        data.read_exact(&mut frame)?;
        let frame = i32::from_le_bytes(frame);

        let mut hold = [0u8; 1];
        data.read_exact(&mut hold)?;
        let hold = hold[0] == 0x31;

        let mut player_1 = [0u8; 1];
        data.read_exact(&mut player_1)?;
        let player_1 = player_1[0] == 0x31;

        Ok(Self {
            frame,
            hold,
            player_1,
        })
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let bytes = vec![
            (self.frame & 0xFF) as u8,
            ((self.frame >> 8) & 0xFF) as u8,
            ((self.frame >> 16) & 0xFF) as u8,
            ((self.frame >> 24) & 0xFF) as u8,
            if self.hold { 0x31 } else { 0x30 },
            if self.player_1 { 0x31 } else { 0x30 },
        ];

        bytes
    }
}

impl ReplayFormat for ZBotReplay {
    type ClickType = ZBotClick;

    fn new(fps: f32) -> Self {
        Self {
            fps,
            clicks: vec![],
        }
    }

    fn add_click(&mut self, click: Self::ClickType) -> () {
        self.clicks.push(click);
    }

    fn dump(&self) -> eyre::Result<Vec<u8>> {
        let mut bytes = vec![];
        
        let delta: f32 = 1.0 / self.fps; // This is disgusting, I hate it, what the hell Fig
        bytes.extend_from_slice(&delta.to_le_bytes());
        bytes.extend_from_slice(&(1.0 as f32).to_le_bytes());

        for click in &self.clicks {
            bytes.extend_from_slice(&click.to_binary());
        }

        Ok(bytes)
    }

    fn from_data(data: &mut std::io::Cursor<Vec<u8>>) -> eyre::Result<Self>
        where
            Self: Sized {
        let mut delta = [0u8; 4];
        data.read_exact(&mut delta)?;
        let delta = f32::from_le_bytes(delta);

        let mut speedhack = [0u8; 4];
        data.read_exact(&mut speedhack)?;
        let speedhack = f32::from_le_bytes(speedhack);

        let fps = (1.0 / (delta * speedhack)).round();

        let mut clicks = Vec::new();

        while data.position() < data.get_ref().len() as u64 {
            clicks.push(ZBotClick::from_binary(data)?);
        }

        Ok(Self {
            fps,
            clicks,
        })
    }

    fn from_universal(replay: super::replay::Replay) -> eyre::Result<Self>
        where
            Self: Sized {
        if replay.game_version != GameVersion::Version2113 {
            return Err(eyre::eyre!("Unsupported game version: {:?}", replay.game_version));
        }

        let mut zbot_replay = Self::new(replay.fps);

        zbot_replay.add_click(ZBotClick {
            frame: 0,
            hold: false,
            player_1: true,
        });
        zbot_replay.add_click(ZBotClick {
            frame: 0,
            hold: false,
            player_1: false,
        });

        replay.clicks.iter().for_each(|click| {

            let frame = if click.frame == 0 { 1 } else { click.frame as i32 };

            if click.p1 != ReplayClickType::Skip {
                zbot_replay.add_click(ZBotClick {
                    frame,
                    hold: click.p1 == ReplayClickType::Click,
                    player_1: true,
                });
            }

            if click.p2 != ReplayClickType::Skip {
                zbot_replay.add_click(ZBotClick {
                    frame,
                    hold: click.p2 == ReplayClickType::Click,
                    player_1: false,
                });
            }
        });

        Ok(zbot_replay)
    }

    fn to_universal(&self) -> eyre::Result<super::replay::Replay> {
        let mut replay = super::replay::Replay::new(self.fps, GameVersion::Version2113);

        for click in self.clicks.iter() {
            if click.player_1 {
                replay.clicks.push(super::replay::ReplayClick {
                    frame: click.frame as i64,
                    p1: if click.hold { ReplayClickType::Click } else { ReplayClickType::Release },
                    p2: ReplayClickType::Skip,
                });
            } else {
                replay.clicks.push(super::replay::ReplayClick {
                    frame: click.frame as i64,
                    p1: ReplayClickType::Skip,
                    p2: if click.hold { ReplayClickType::Click } else { ReplayClickType::Release },
                });
            }
        }

        Ok(replay)
    }

    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self>
        where
            Self: Sized {
        let mut file = std::fs::File::open(path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let mut cursor = std::io::Cursor::new(data);
        Self::from_data(&mut cursor)
    }

    fn save(&self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
        let mut file = std::fs::File::create(path)?;

        file.write_all(&self.dump()?)?;

        Ok(())
    }
}
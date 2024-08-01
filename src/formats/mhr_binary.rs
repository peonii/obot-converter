use std::io::{Cursor, Read, Seek};

use super::replay::{GameVersion, Replay, ReplayFormat};

pub struct MHRBinaryReplay {
    pub fps: f64,
    pub clicks: Vec<MHRBinaryClick>,
}

pub struct MHRBinaryClick {
    pub frame: i32,
    pub p1: bool,
    pub p2: bool,

    /// How large the click is with additional metadata.
    size: u32
}

impl MHRBinaryClick {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![
            1 as u8,
            0 as u8,
            if self.p1 { 1 as u8 } else { 0 as u8 },
            if self.p2 { 1 as u8 } else { 0 as u8 },
            (self.frame & 0xFF) as u8,
            ((self.frame >> 8) & 0xFF) as u8,
            ((self.frame >> 16) & 0xFF) as u8,
            ((self.frame >> 24) & 0xFF) as u8,
        ];

        for _ in 0..(self.size - 8) {
            bytes.push(0);
        }

        bytes
    }

    pub fn from_binary(size: u32, data: &mut Cursor<Vec<u8>>) -> eyre::Result<Self> {
        data.seek(std::io::SeekFrom::Current(2))?;

        let mut p1 = [0; 1];
        data.read_exact(&mut p1)?;
        let p1 = p1[0] == 1;

        let mut p2 = [0; 1];
        data.read_exact(&mut p2)?;
        let p2 = p2[0] == 1;

        let mut frame = [0; 4];
        data.read_exact(&mut frame)?;
        let frame = i32::from_le_bytes(frame);

        data.seek(std::io::SeekFrom::Current((size - 8) as i64))?;

        Ok(Self {
            frame,
            p1,
            p2,
            size,
        })
    }
}

static MHR_BINARY_HEADER: [u8; 8] = [0x48, 0x41, 0x43, 0x4B, 0x50, 0x52, 0x4F, 0x07];

impl ReplayFormat for MHRBinaryReplay {
    type ClickType = MHRBinaryClick;

    fn new(fps: f32) -> Self {
        Self {
            fps: fps.into(),
            clicks: Vec::new(),
        }
    }

    fn from_data(data: &mut Cursor<Vec<u8>>) -> eyre::Result<Self>
        where
            Self: Sized {
        let mut replay = MHRBinaryReplay::new(60.0);

        let mut header = [0; 8];
        data.read_exact(&mut header)?;

        if header != MHR_BINARY_HEADER {
            return Err(eyre::eyre!("Invalid header"));
        }

        // Should be safely ignore-able?
        let mut meta_size = [0; 4];
        data.read_exact(&mut meta_size)?;
        let meta_size = i32::from_le_bytes(meta_size);

        let mut fps = [0; 4];
        data.read_exact(&mut fps)?;
        let actual_fps = i32::from_le_bytes(fps);

        replay.fps = actual_fps as f64;

        // Skip 8 bytes
        data.seek(std::io::SeekFrom::Current(8))?;
        
        let mut event_size = [0; 4];
        data.read_exact(&mut event_size)?;
        let event_size = u32::from_le_bytes(event_size);

        let mut event_count = [0; 4];
        data.read_exact(&mut event_count)?;
        let event_count = u32::from_le_bytes(event_count);

        for _ in 0..event_count {
            let click = MHRBinaryClick::from_binary(event_size as u32, data)?;
            replay.clicks.push(click);
        }

        Ok(replay)
    }

    fn add_click(&mut self, click: Self::ClickType) -> () {
        self.clicks.push(click);
    }

    fn dump(&self) -> eyre::Result<Vec<u8>> {
        let mut data = Vec::new();
        data.extend_from_slice(&MHR_BINARY_HEADER);
        data.extend_from_slice(&(4 as i32).to_le_bytes()); // Placeholder for meta size
        let fps = self.fps as i32;
        data.extend_from_slice(&fps.to_le_bytes());
        data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]); // Placeholder for reserved

        let first_click = self.clicks.first().unwrap();
        let event_size = first_click.size;

        data.extend_from_slice(&event_size.to_le_bytes()); // Placeholder for event size
        data.extend_from_slice(&(self.clicks.len() as i32).to_le_bytes());

        for click in self.clicks.iter() {
            data.extend_from_slice(&click.to_binary());
        }

        data.extend_from_slice(&[0xFA, 0x67, 0x55, 0x5A, 0x8D, 0x95, 0x94, 0x07, 0xC9, 0x8C, 0xBA, 0x7F, 0x75, 0x9C, 0xEF, 0x3C]);

        Ok(data)
    }

    fn from_universal(replay: Replay) -> eyre::Result<Self>
        where
            Self: Sized {
        if replay.game_version != GameVersion::Version2113 {
            return Err(eyre::eyre!("Unsupported game version: {:?}", replay.game_version));
        }

        let mut mhr_replay = MHRBinaryReplay::new(replay.fps as f32);

        for click in replay.clicks.iter() {
            if click.p1 == super::replay::ReplayClickType::Click {
                mhr_replay.add_click(MHRBinaryClick {
                    frame: click.frame as i32,
                    p1: true,
                    p2: false,
                    size: 32,
                });
            } else if click.p1 == super::replay::ReplayClickType::Release {
                mhr_replay.add_click(MHRBinaryClick {
                    frame: click.frame as i32,
                    p1: false,
                    p2: false,
                    size: 32,
                });
            }

            if click.p2 == super::replay::ReplayClickType::Click {
                mhr_replay.add_click(MHRBinaryClick {
                    frame: click.frame as i32,
                    p1: true,
                    p2: true,
                    size: 32,
                });
            } else if click.p2 == super::replay::ReplayClickType::Release {
                mhr_replay.add_click(MHRBinaryClick {
                    frame: click.frame as i32,
                    p1: false,
                    p2: true,
                    size: 32,
                });
            }
        }

        Ok(mhr_replay)
    }

    fn to_universal(&self) -> eyre::Result<Replay> {
        let mut replay = Replay::new(self.fps as f32, GameVersion::Version2113);

        for click in self.clicks.iter() {
            if click.p2 {
                replay.clicks.push(
                    super::replay::ReplayClick {
                        frame: click.frame as i64,
                        p1: super::replay::ReplayClickType::Skip,
                        p2: if click.p1 { super::replay::ReplayClickType::Click } else { super::replay::ReplayClickType::Release },
                    }
                )
            } else {
                replay.clicks.push(
                    super::replay::ReplayClick {
                        frame: click.frame as i64,
                        p1: if click.p1 { super::replay::ReplayClickType::Click } else { super::replay::ReplayClickType::Release },
                        p2: super::replay::ReplayClickType::Skip,
                    }
                )
            }
            
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

mod tests {
    use super::*;

    #[test]
    fn test_click_to_binary() {
        let click = MHRBinaryClick {
            frame: 0x12345678,
            p1: true,
            p2: false,
            size: 8,
        };

        let binary = click.to_binary();
        assert_eq!(binary, vec![1, 0, 1, 0, 0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_click_padding() {
        let click = MHRBinaryClick {
            frame: 0x12345678,
            p1: true,
            p2: false,
            size: 16,
        };

        let binary = click.to_binary();
        assert_eq!(binary, vec![1, 0, 1, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
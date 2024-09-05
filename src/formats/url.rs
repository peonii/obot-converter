use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};

#[derive(PartialEq)]
enum URLReplayType {
    XPos,
    Frames,
    Both,
}

impl TryFrom<u8> for URLReplayType {
    type Error = ReplayError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::XPos),
            1 => Ok(Self::Frames),
            2 => Ok(Self::Both),
            _ => Err(ReplayError::ParseError),
        }
    }
}

impl From<URLReplayType> for u8 {
    fn from(value: URLReplayType) -> Self {
        match value {
            URLReplayType::XPos => 0,
            URLReplayType::Frames => 1,
            URLReplayType::Both => 2,
        }
    }
}

impl Replay {
    pub fn parse_url(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut reader = BufReader::new(reader);

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        self.fps = f32::from_le_bytes(buf);
        self.game_version = GameVersion::Version2113;

        let mut small_buf = [0u8; 1];
        reader.read_exact(&mut small_buf)?;
        let replay_type: URLReplayType = small_buf[0].try_into()?;

        let click_size: u64 = if replay_type == URLReplayType::Both {
            9
        } else {
            5
        };

        let old_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))?;
        if old_pos != len {
            reader.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        let clicks_len = (len - 5) / click_size;

        self.clicks.reserve(clicks_len as usize);

        for _ in 0..clicks_len {
            reader.read_exact(&mut small_buf)?;
            let state = small_buf[0];

            let hold = state & 1 == 1;
            let player_2 = state >> 1 == 1;

            let frame = match replay_type {
                URLReplayType::XPos => return Err(ReplayError::ParseError),
                URLReplayType::Frames => {
                    reader.read_exact(&mut buf)?;
                    u32::from_le_bytes(buf)
                }
                URLReplayType::Both => {
                    reader.seek(std::io::SeekFrom::Current(4))?;
                    reader.read_exact(&mut buf)?;
                    u32::from_le_bytes(buf)
                }
            };

            self.clicks.push(Click::from_hold(frame, hold, player_2));
        }

        Ok(())
    }

    pub fn write_url(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write_all(&self.fps.to_le_bytes())?;
        writer.write_all(&[URLReplayType::Frames.into()])?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                let state: u8 = (hold as u8) | if p2 { 2 } else { 0 };

                writer.write_all(&state.to_le_bytes())?;
                writer.write_all(&frame.to_le_bytes())?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

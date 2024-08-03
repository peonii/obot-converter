use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, Replay, ReplayError};

#[derive(PartialEq)]
enum URLReplayType {
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

impl From<URLReplayType> for u8 {
    fn from(value: URLReplayType) -> u8 {
        match value {
            URLReplayType::XPos => 0,
            URLReplayType::Frames => 1,
            URLReplayType::Both => 2
        }
    }
}


impl Replay {
    pub fn parse_url(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut reader = BufReader::new(reader);

        let mut buf = [0u8; 4];
        reader.read(&mut buf)?;
        self.fps = f32::from_le_bytes(buf);

        let mut small_buf = [0u8; 1];
        reader.read(&mut small_buf)?;
        let replay_type: URLReplayType = small_buf[0].into();

        let click_size: u64 = if replay_type == URLReplayType::Both { 9 } else { 5 };
        let clicks_len = (reader.stream_len()? - 5) / click_size;

        self.clicks.reserve(clicks_len as usize);

        for _ in 0..clicks_len {
            reader.read(&mut small_buf)?;
            let state = small_buf[0];

            let hold = state & 1 == 1;
            let player_2 = state >> 1 == 1;

            let frame = match replay_type {
                URLReplayType::XPos => return Err(ReplayError::ParseError),
                URLReplayType::Frames => {
                    reader.read(&mut buf)?;
                    u32::from_le_bytes(buf)
                }
                URLReplayType::Both => {
                    reader.seek(std::io::SeekFrom::Current(4))?;
                    reader.read(&mut buf)?;
                    u32::from_le_bytes(buf)
                }
            };

            self.clicks.push(Click::from_hold(frame, hold, player_2));
        }

        Ok(())
    }

    pub fn write_url(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write(&self.fps.to_le_bytes())?;
        writer.write(&[URLReplayType::Frames.into()])?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                let state: u8 = if hold { 1 } else { 0 } | if p2 { 2 } else { 0 };

                writer.write(&state.to_le_bytes())?;
                writer.write(&frame.to_le_bytes())?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

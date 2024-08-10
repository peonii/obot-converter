use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};

static FEMBOT_HEADER: [u8; 4] = [0x46, 0x42, 0x52, 0x50];

impl Replay {
    pub fn parse_fembot(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        self.game_version = GameVersion::Version2113;

        let mut reader = BufReader::new(reader);

        let mut small_buf = [0u8; 1];
        let mut buf = [0u8; 4];

        reader.read(&mut buf)?;
        if buf != FEMBOT_HEADER {
            return Err(ReplayError::ParseError);
        }

        reader.read(&mut buf)?;
        self.fps = f32::from_le_bytes(buf);

        let old_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))?;
        if old_pos != len {
            reader.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        let clicks_len = len / 6;

        self.clicks.reserve(clicks_len as usize);
        for _ in 0..clicks_len {
            reader.read(&mut small_buf)?;
            let state = u8::from_le_bytes(small_buf);

            let hold = state & 1 == 1;
            let p2 = state & 2 == 2;

            reader.read(&mut buf)?;
            let frame = u32::from_le_bytes(buf);

            reader.seek(std::io::SeekFrom::Current(60))?;

            self.clicks.push(Click::from_hold(frame, hold, p2));
        }

        Ok(())
    }

    pub fn write_fembot(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write(&FEMBOT_HEADER)?;
        writer.write(&self.fps.to_le_bytes())?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write(&(if hold { 1u8 } else { 0u8 } | if p2 { 2u8 } else { 0u8 }).to_le_bytes())?;
                writer.write(&frame.to_le_bytes())?;
                writer.write(&[0u8; 60])?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}
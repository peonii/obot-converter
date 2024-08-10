use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};


impl Replay {
    pub fn parse_rush(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        self.game_version = GameVersion::Version2113;

        let mut reader = BufReader::new(reader);

        let mut small_buf = [0u8; 1];
        let mut mid_buf = [0u8; 2];
        let mut buf = [0u8; 4];

        reader.read(&mut mid_buf)?;
        self.fps = i16::from_le_bytes(mid_buf) as f32;

        let old_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))?;
        if old_pos != len {
            reader.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        let clicks_len = len / 5;

        self.clicks.reserve(clicks_len as usize);
        for _ in 0..clicks_len {
            reader.read(&mut buf)?;
            let frame = u32::from_le_bytes(buf);

            reader.read(&mut small_buf)?;
            let hold = (small_buf[0] & 0x1) == 1;
            let player_2 = (small_buf[0] & 0x2) == 2;

            self.clicks.push(Click::from_hold(frame, hold, player_2));
        }

        Ok(())
    }

    pub fn write_rush(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write(&(self.fps as i16).to_le_bytes())?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write(&frame.to_le_bytes())?;
                let state: u8 = if hold { 1 } else { 0 } | if p2 { 0x2 } else { 0 };
                writer.write(&state.to_le_bytes())?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}
use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};

impl Replay {
    pub fn parse_kdbot(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        self.game_version = GameVersion::Version2113;

        let mut reader = BufReader::new(reader);

        let mut small_buf = [0u8; 1];
        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        self.fps = f32::from_le_bytes(buf);

        let old_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))?;
        if old_pos != len {
            reader.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        let clicks_len = len / 6;

        self.clicks.reserve(clicks_len as usize);
        for _ in 0..clicks_len {
            reader.read_exact(&mut buf)?;
            let frame = u32::from_le_bytes(buf);

            reader.read_exact(&mut small_buf)?;
            let hold = small_buf[0] == 1;

            reader.read_exact(&mut small_buf)?;
            let player_2 = small_buf[0] == 1;

            self.clicks.push(Click::from_hold(frame, hold, player_2));
        }

        Ok(())
    }

    pub fn write_kdbot(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write_all(&self.fps.to_le_bytes())?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write_all(&frame.to_le_bytes())?;
                writer.write_all(&(hold as u8).to_le_bytes())?;
                writer.write_all(&(p2 as u8).to_le_bytes())?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

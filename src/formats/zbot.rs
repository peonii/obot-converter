use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};


impl Replay {
    pub fn parse_zbot(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        self.clear();

        let mut reader = BufReader::new(reader);

        let mut buf = [0u8; 4];

        reader.read(&mut buf)?;
        let delta = f32::from_le_bytes(buf);
        reader.read(&mut buf)?;
        let speedhack = f32::from_le_bytes(buf);

        self.fps = (1.0 / (delta * speedhack)).round();

        let old_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))?;
        if old_pos != len {
            reader.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        // 8 is how much space the delta and speedhack take up, 
        // 6 is how much space one click takes up (in bytes)
        let clicks_len = (len - 8) / 6; 
        self.clicks.reserve(clicks_len as usize);
        self.game_version = GameVersion::Version2113;

        // Preallocate memory for reading hold and player 2 data
        let mut small_buf = [0u8; 1];

        for _ in 0..clicks_len {
            reader.read(&mut buf)?;
            let frame = i32::from_le_bytes(buf);

            reader.read(&mut small_buf)?;
            let hold = small_buf[0] == 0x31;

            reader.read(&mut small_buf)?;
            let player_2 = small_buf[0] == 0x31;
            
            self.clicks.push(
                Click::from_hold(frame as u32, hold, player_2)
            );
        }

        Ok(())
    }   

    pub fn write_zbot(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write(&((1.0 / self.fps).to_le_bytes()))?; // Delta
        writer.write(&(1.0_f32.to_le_bytes()))?; // Speedhack

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write(&(frame as i32).to_le_bytes())?;
                writer.write(
                    &(if hold { 0x31_u8 } else { 0x30_u8 }).to_le_bytes()
                )?;
                writer.write(
                    &(if !p2 { 0x31_u8 } else { 0x30_u8 }).to_le_bytes()
                )?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

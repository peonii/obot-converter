use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};

impl Replay {
    pub fn parse_silicate(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        self.game_version = GameVersion::Version2113;

        let mut reader = BufReader::new(reader);

        let mut fps = [0; 8];
        reader.read_exact(&mut fps)?;
        self.fps = f64::from_le_bytes(fps) as f32;

        let mut input_count = [0; 4];
        reader.read_exact(&mut input_count)?;
        let input_count = u32::from_le_bytes(input_count);

        self.clicks.reserve(input_count as usize);

        for _ in 0..input_count {
            let mut state = [0; 4];
            reader.read_exact(&mut state)?;

            let state = u32::from_le_bytes(state);

            let frame = (state >> 4) as u32;
            let player_2 = (state & 0b1000) != 0;
            match (state & 0b0110) >> 1 {
                1 => {},
                _ => { continue; }
            };

            let down = (state & 0b0001) != 0;

            self.clicks.push(
                Click::from_hold(frame, down, player_2)
            );
        }


        Ok(())
    }

    pub fn write_silicate(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write_all(&self.fps.to_le_bytes())?;
        writer.write_all(&(self.clicks.len() as u32).to_le_bytes())?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                let mut state: u32 = 0;

                state |= frame << 4;
                state |= if p2 { 1 } else { 0 } << 3;
                state |= 1 << 1;
                state |= if hold { 1 } else { 0 };

                writer.write_all(&state.to_le_bytes())?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

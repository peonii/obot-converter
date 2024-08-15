use super::replay::{Click, GameVersion, Replay, ReplayError};
use std::io::{BufRead, Read, Seek, Write};

impl Replay {
    pub fn parse_xbot(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let reader = std::io::BufReader::new(reader);
        let mut lines = reader.lines();

        self.game_version = GameVersion::Version2113;
        self.fps = lines
            .next()
            .ok_or(ReplayError::ParseError)?
            .map_err(|_| ReplayError::ParseError)?
            .split(' ')
            .nth(1)
            .ok_or(ReplayError::ParseError)?
            .parse::<f32>()
            .map_err(|_| ReplayError::ParseError)?;

        lines.skip(1).try_for_each(|line| {
            let line = line.map_err(|_| ReplayError::ParseError)?;
            let mut data = line.split_whitespace();

            let state = data
                .next()
                .ok_or(ReplayError::ParseError)?
                .parse::<u32>()
                .map_err(|_| ReplayError::ParseError)?;

            let hold = state % 2 == 1;
            let p2 = state > 1;

            let frame = data
                .next()
                .ok_or(ReplayError::ParseError)?
                .parse::<u32>()
                .map_err(|_| ReplayError::ParseError)?;

            self.clicks.push(Click::from_hold(frame, hold, p2));

            Ok::<(), ReplayError>(())
        })?;

        Ok(())
    }

    pub fn write_xbot(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = std::io::BufWriter::new(writer);

        writer
            .write_all(format!("fps: {}\n", self.fps).as_bytes())
            .map_err(|_| ReplayError::WriteError)?;
        writer.write_all(b"frames\n")?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write_all(
                    format!("{} {}\n", (hold as u8) | if p2 { 2 } else { 0 }, frame,).as_bytes(),
                )?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

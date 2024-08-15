use std::io::{BufReader, BufWriter, Read, Seek, Write};

use ybot_fmt::{Action, Meta, PlayerButton, TimedAction};

use super::replay::{Click, GameVersion, Replay, ReplayError};

static YBOT1_HEADER: [u8; 4] = [0x79, 0x62, 0x6F, 0x74];

impl Replay {
    pub fn parse_ybot2(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut replay = ybot_fmt::Macro::open(reader)?;

        self.fps = replay.get(Meta::FPS)?;
        self.game_version = GameVersion::Version2206;

        let mut frame = 0;

        // Apparently I can't trust yBot 2's PRESSES meta field
        // self.clicks.reserve(
        //     replay.get(Meta::PRESSES)? as usize
        // );

        for action in replay.actions() {
            let action = action?;

            frame += action.delta;

            match action.action {
                Action::Button(p1, hold, b) => {
                    if b != PlayerButton::Jump {
                        continue;
                    }

                    self.clicks.push(Click::from_hold(frame as u32, hold, !p1));
                }
                Action::FPS(_) => {
                    todo!();
                }
            }
        }

        Ok(())
    }

    pub fn write_ybot2(&self, writer: &mut (impl Read + Write + Seek)) -> Result<(), ReplayError> {
        let mut replay = ybot_fmt::Macro::create(writer)?;

        replay.set(Meta::FPS, self.fps)?;
        replay.set(Meta::PRESSES, self.clicks.len() as u64)?;
        replay.set(Meta::TOTAL_PRESSES, self.clicks.len() as u64)?;

        let mut last_frame = 0;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                let delta = frame - last_frame;
                last_frame = frame;

                replay.add(TimedAction::new(
                    delta.into(),
                    Action::Button(!p2, hold, PlayerButton::Jump),
                ))?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }

    pub fn parse_ybot1(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut reader = BufReader::new(reader);

        self.game_version = GameVersion::Version2113;

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        if buf != YBOT1_HEADER {
            return Err(ReplayError::ParseError);
        }

        reader.read_exact(&mut buf)?;
        self.fps = f32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let clicks_len = i32::from_le_bytes(buf);
        self.clicks.reserve(clicks_len as usize);

        for _ in 0..clicks_len {
            reader.read_exact(&mut buf)?;
            let frame = u32::from_le_bytes(buf);

            reader.read_exact(&mut buf)?;
            let state = u32::from_le_bytes(buf);

            let hold = (state & 2) == 2;
            let p2 = (state & 1) == 1;

            self.clicks.push(Click::from_hold(frame, hold, p2));
        }

        Ok(())
    }

    pub fn write_ybot1(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write_all(&YBOT1_HEADER)?;
        writer.write_all(&self.fps.to_le_bytes())?;
        writer.write_all(&self.clicks.len().to_le_bytes())?;

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write_all(&frame.to_le_bytes())?;
                let state: u32 = if hold { 2 } else { 0 } | (p2 as u32);
                writer.write_all(&state.to_le_bytes())?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}

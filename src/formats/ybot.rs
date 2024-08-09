use std::io::{Read, Seek, Write};

use ybot_fmt::{Action, Meta, PlayerButton, TimedAction};

use super::replay::{Click, GameVersion, Replay, ReplayError};


impl Replay {
    pub fn parse_ybot2(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut replay = ybot_fmt::Macro::open(reader)?;

        self.fps = replay.get(Meta::FPS)?;
        self.game_version = GameVersion::Version2206;

        let mut frame = 0;

        self.clicks.reserve(
            replay.get(Meta::PRESSES)? as usize
        );

        for action in replay.actions() {
            let action = action?;

            frame += action.delta;

            match action.action {
                Action::Button(p1, hold, _) => {
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

                replay.add(TimedAction::new(delta.into(), Action::Button(!p2, hold, PlayerButton::Jump)))?;

                Ok::<(), ReplayError>(())
            })
        })?;

        Ok(())
    }
}
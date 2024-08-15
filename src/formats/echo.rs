use serde::{Deserialize, Serialize};

use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};

#[derive(Serialize, Deserialize)]
struct EchoOldReplay {
    #[serde(rename = "FPS")]
    pub fps: f32,

    #[serde(rename = "Starting Frame")]
    pub start_frame: u32,

    #[serde(rename = "Echo Replay")]
    pub clicks: Vec<EchoOldClick>,
}

#[derive(Serialize, Deserialize)]
struct EchoOldClick {
    #[serde(rename = "Hold")]
    pub hold: bool,
    #[serde(rename = "Player 2")]
    pub p2: bool,
    #[serde(rename = "Frame")]
    pub frame: u32,
    #[serde(rename = "X Position")]
    pub xpos: f32,
}

#[derive(Serialize, Deserialize)]
struct EchoNewReplay {
    pub fps: f32,
    pub inputs: Vec<EchoNewClick>,
}

#[derive(Serialize, Deserialize)]
struct EchoNewClick {
    #[serde(rename = "holding")]
    pub hold: bool,
    #[serde(rename = "player_2", skip_serializing_if = "Option::is_none")]
    pub p2: Option<bool>,
    pub frame: u32,
}

impl From<EchoOldClick> for Click {
    fn from(value: EchoOldClick) -> Self {
        Self::from_hold(value.frame, value.hold, value.p2)
    }
}

impl From<EchoNewClick> for Click {
    fn from(value: EchoNewClick) -> Self {
        Self::from_hold(value.frame, value.hold, value.p2.unwrap_or(false))
    }
}

static ECHO_BIN_HEADER: [u8; 4] = [0x4D, 0x45, 0x54, 0x41];
static ECHO_FULL_HEADER: [u8; 4] = [0x44, 0x42, 0x47, 0x00];

impl Replay {
    pub fn parse_echo_bin(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        self.game_version = GameVersion::Version2113;

        let mut reader = BufReader::new(reader);

        let mut small_buf = [0u8; 1];
        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        if buf != ECHO_BIN_HEADER {
            return Err(ReplayError::ParseError);
        }

        reader.read_exact(&mut buf)?;
        let action_size = if buf == ECHO_FULL_HEADER { 34 } else { 6 };

        reader.seek(std::io::SeekFrom::Start(24))?;
        reader.read_exact(&mut buf)?;
        self.fps = f32::from_le_bytes(buf);

        reader.seek(std::io::SeekFrom::Start(48))?;

        let old_pos = reader.stream_position()?;
        let len = reader.seek(std::io::SeekFrom::End(0))?;
        if old_pos != len {
            reader.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        let clicks_len = len / action_size;
        self.clicks.reserve(clicks_len as usize);
        for _ in 0..clicks_len {
            reader.read_exact(&mut buf)?;
            let frame = u32::from_le_bytes(buf);

            reader.read_exact(&mut small_buf)?;
            let down = small_buf[0] == 1;
            reader.read_exact(&mut small_buf)?;
            let p2 = small_buf[0] == 1;

            if action_size == 34 {
                reader.seek(std::io::SeekFrom::Current(28))?;
            }

            self.clicks.push(Click::from_hold(frame, down, p2));
        }

        Ok(())
    }

    pub fn parse_echo_new(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let replay: EchoNewReplay =
            serde_json::from_reader(reader).map_err(|_| ReplayError::ParseError)?;

        self.fps = replay.fps.round();
        self.clicks = replay.inputs.into_iter().map(EchoNewClick::into).collect();
        self.game_version = GameVersion::Version2113;

        Ok(())
    }

    pub fn parse_echo_old(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let replay: EchoOldReplay =
            serde_json::from_reader(reader).map_err(|_| ReplayError::ParseError)?;

        self.fps = replay.fps.round();
        self.clicks = replay
            .clicks
            .into_iter()
            .map(|click| EchoOldClick {
                frame: click.frame + replay.start_frame,
                ..click
            })
            .map(EchoOldClick::into)
            .collect();
        self.game_version = GameVersion::Version2113;

        Ok(())
    }

    pub fn write_echo_bin(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write_all(&ECHO_BIN_HEADER)?;
        writer.write_all(&[0u8; 20])?;
        writer.write_all(&self.fps.to_le_bytes())?;
        writer.write_all(&[0u8; 20])?;

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

    pub fn write_echo_new(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut replay = EchoNewReplay {
            fps: self.fps,
            inputs: vec![],
        };

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                replay.inputs.push(EchoNewClick {
                    frame,
                    hold,
                    p2: if p2 { Some(p2) } else { None },
                });

                Ok::<(), ReplayError>(())
            })
        })?;

        if self.settings.beautified_json {
            serde_json::to_writer_pretty(writer, &replay).map_err(|_| ReplayError::WriteError)?;
        } else {
            serde_json::to_writer(writer, &replay).map_err(|_| ReplayError::WriteError)?;
        }

        Ok(())
    }

    pub fn write_echo_old(&self, writer: &mut (impl Write + Seek)) -> Result<(), ReplayError> {
        let mut replay = EchoOldReplay {
            fps: self.fps,
            start_frame: 0,
            clicks: vec![],
        };

        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                replay.clicks.push(EchoOldClick {
                    frame,
                    hold,
                    p2,
                    xpos: 0.0,
                });

                Ok::<(), ReplayError>(())
            })
        })?;

        if self.settings.beautified_json {
            serde_json::to_writer_pretty(writer, &replay).map_err(|_| ReplayError::WriteError)?;
        } else {
            serde_json::to_writer(writer, &replay).map_err(|_| ReplayError::WriteError)?;
        }

        Ok(())
    }
}

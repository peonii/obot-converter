use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, GameVersion, Replay, ReplayError};

static MHR_BINARY_HEADER: [u8; 8] = [0x48, 0x41, 0x43, 0x4B, 0x50, 0x52, 0x4F, 0x07];
static MHR_BINARY_FOOTER: [u8; 16] = [
    0xFA, 0x67, 0x55, 0x5A, 0x8D, 0x95, 0x94, 0x07, 0xC9, 0x8C, 0xBA, 0x7F, 0x75, 0x9C, 0xEF, 0x3C,
];

impl Replay {
    pub fn parse_mhr_binary(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut reader = BufReader::new(reader);

        self.game_version = GameVersion::Version2113;

        let mut header_buf = [0u8; 8];
        reader.read_exact(&mut header_buf)?;
        if header_buf != MHR_BINARY_HEADER {
            return Err(ReplayError::ParseError);
        }

        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        let meta_size = i32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        self.fps = i32::from_le_bytes(buf) as f32;

        reader.seek(std::io::SeekFrom::Current((meta_size - 4) as i64))?;

        // Skip reserved space (ABSOLUTE in replays created by MHR)
        reader.seek(std::io::SeekFrom::Current(8))?;

        reader.read_exact(&mut buf)?;
        let event_size = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let event_count = u32::from_le_bytes(buf);

        self.clicks.reserve(event_count as usize);

        let mut small_buf = [0u8; 1];
        for _ in 0..event_count {
            reader.seek(std::io::SeekFrom::Current(2))?;

            reader.read_exact(&mut small_buf)?;
            let hold = small_buf[0] == 1;

            reader.read_exact(&mut small_buf)?;
            let p2 = small_buf[0] == 1;

            reader.read_exact(&mut buf)?;
            let frame = i32::from_le_bytes(buf);

            reader.seek(std::io::SeekFrom::Current((event_size - 8) as i64))?;

            self.clicks.push(Click::from_hold(frame as u32, hold, p2));
        }

        Ok(())
    }

    pub fn write_mhr_binary(&self, writer: impl Write + Seek) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write_all(&MHR_BINARY_HEADER)?; // Header
        writer.write_all(&4_i32.to_le_bytes())?; // Meta size
        writer.write_all(&(self.fps as i32).to_le_bytes())?; // FPS
        writer.write_all(&[0, 0, 0, 0, 0, 0, 0, 0])?; // Reserved
        writer.write_all(&32_u32.to_le_bytes())?; // Event size
                                                  // Event count offset is 0x1c
        writer.write_all(&0_u32.to_le_bytes())?; // Event count

        let mut event_count = 0u32;
        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write_all(&1_u16.to_le_bytes())?;

                writer.write_all(&(hold as u8).to_le_bytes())?;
                writer.write_all(&(p2 as u8).to_le_bytes())?;
                writer.write_all(&(frame as i32).to_le_bytes())?;
                // Fill up for minimum event space
                writer.write_all(&[0u8; 24])?;

                event_count += 1;

                Ok::<(), ReplayError>(())
            })
        })?;

        // Footer
        writer.write_all(&MHR_BINARY_FOOTER)?;

        // Update event count
        writer.seek(std::io::SeekFrom::Start(0x1c))?;
        writer.write_all(&event_count.to_le_bytes())?;

        Ok(())
    }
}

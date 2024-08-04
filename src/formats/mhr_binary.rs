use std::io::{BufReader, BufWriter, Read, Seek, Write};

use super::replay::{Click, Replay, ReplayError};

static MHR_BINARY_HEADER: [u8; 8] = [0x48, 0x41, 0x43, 0x4B, 0x50, 0x52, 0x4F, 0x07];
static MHR_BINARY_FOOTER: [u8; 16] = [0xFA, 0x67, 0x55, 0x5A, 0x8D, 0x95, 0x94, 0x07, 0xC9, 0x8C, 0xBA, 0x7F, 0x75, 0x9C, 0xEF, 0x3C];

impl Replay {
    pub fn parse_mhr_binary(&mut self, reader: impl Read + Seek) -> Result<(), ReplayError> {
        let mut reader = BufReader::new(reader);

        let mut header_buf = [0u8; 8];
        reader.read(&mut header_buf)?;
        if header_buf != MHR_BINARY_HEADER {
            return Err(ReplayError::ParseError);
        }

        let mut buf = [0u8; 4];
        
        reader.read(&mut buf)?;
        let meta_size = i32::from_le_bytes(buf);

        reader.read(&mut buf)?;
        self.fps = i32::from_le_bytes(buf) as f32;

        reader.seek(
            std::io::SeekFrom::Current((meta_size - 4) as i64)
        )?;

        // Skip reserved space (ABSOLUTE in replays created by MHR)
        reader.seek(std::io::SeekFrom::Current(8))?;

        reader.read(&mut buf)?;
        let event_size = u32::from_le_bytes(buf);

        reader.read(&mut buf)?;
        let event_count = u32::from_le_bytes(buf);

        self.clicks.reserve(event_count as usize);

        let mut small_buf = [0u8; 1];
        for _ in 0..event_count {
            reader.seek(std::io::SeekFrom::Current(2))?;

            reader.read(&mut small_buf)?;
            let hold = small_buf[0] == 1;

            reader.read(&mut small_buf)?;
            let p2 = small_buf[0] == 1;

            reader.read(&mut buf)?;
            let frame = i32::from_le_bytes(buf);

            reader.seek(
                std::io::SeekFrom::Current((event_size - 8) as i64)
            )?;

            self.clicks.push(Click::from_hold(frame as u32, hold, p2));
        }

        Ok(())
    }

    pub fn write_mhr_binary(&self, writer: impl Write + Seek) -> Result<(), ReplayError> {
        let mut writer = BufWriter::new(writer);

        writer.write(&MHR_BINARY_HEADER)?; // Header
        writer.write(&4_i32.to_le_bytes())?; // Meta size
        writer.write(&(self.fps as i32).to_le_bytes())?; // FPS
        writer.write(&[0, 0, 0, 0, 0, 0, 0, 0])?; // Reserved
        writer.write(&32_u32.to_le_bytes())?; // Event size
        // Event count offset is 0x1c
        writer.write(&0_u32.to_le_bytes())?; // Event count

        let mut event_count = 0u32;
        self.clicks.iter().try_for_each(|click| {
            click.apply_hold(|frame, hold, p2| {
                writer.write(&1_u16.to_le_bytes())?;

                writer.write(
                    &(if hold { 1_u8 } else { 0_u8 }).to_le_bytes()
                )?;
                writer.write(
                    &(if p2 { 1_u8 } else { 0_u8 }).to_le_bytes()
                )?;
                writer.write(&(frame as i32).to_le_bytes())?;
                // Fill up for minimum event space
                writer.write(&[0u8; 24])?;

                event_count += 1;

                Ok::<(), ReplayError>(())
            })
        })?;

        // Footer
        writer.write(&MHR_BINARY_FOOTER)?;

        // Update event count
        writer.seek(std::io::SeekFrom::Start(0x1c))?;
        writer.write(&event_count.to_le_bytes())?;

        Ok(())
    }
}
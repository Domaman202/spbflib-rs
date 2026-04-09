use crate::raw::{str_to_bytes_unaligned_small, usize_to_u16, usize_to_u32};
use crate::write::{SPBFWriter, SPBFWriterError, SPBFWriterWriteError};
use crate::{SPBFType, SPBFVersion};
use std::io::Write;

pub struct RawWriterSmallUnalignedLittleEndian;

impl RawWriterSmallUnalignedLittleEndian {
    pub fn write(writer: &mut SPBFWriter) -> Result<Vec<u8>, SPBFWriterError> {
        let mut out = Vec::new();

        out.write(b".SPBF\0")?;
        out.write(&u8::to_le_bytes(SPBFType::SmallUnalignedLittleEndian.into()))?;
        out.write(&u8::to_le_bytes(SPBFVersion::LAST_SUPPORTED.into()))?;
        out.write(&[0; 4])?; // FORMAT_ENTRY
        out.write(&[0; 4])?; // DATA_ENTRY
        let name = str_to_bytes_unaligned_small(writer.build_name.clone());
        let name = if let Ok(ok) = name { ok } else { return Err(SPBFWriterWriteError::InvalidNameString.into()) };
        out.write(&name)?;
        let version = str_to_bytes_unaligned_small(writer.build_version.clone());
        let version = if let Ok(ok) = version { ok } else { return Err(SPBFWriterWriteError::InvalidVersionString.into()) };
        out.write(&version)?;

        let mut last_offset = 0x10 + name.len() + version.len();

        if !writer.data_formats.is_empty() {
            let mut next_write_addr = 0x8..0xC; // FORMAT_ENTRY

            for format in &writer.data_formats {
                let block_start = u32::to_le_bytes(usize_to_u32(last_offset, SPBFWriterWriteError::InvalidOffset.into())?);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x4; // NEXT

                out.write(&[0; 4])?; // NEXT
                out.write(&u16::to_le_bytes(format.data_id))?;
                let name = str_to_bytes_unaligned_small(format.name.clone());
                let name = if let Ok(ok) = name { ok } else { return Err(SPBFWriterWriteError::InvalidFormatNameLength.into()) };
                out.write(&name)?;

                last_offset += 0x6 + name.len();
            }
        }

        if !writer.data.is_empty() {
            let mut next_write_addr = 0xC..0x10; // DATA_ENTRY

            for data in &writer.data {
                let block_start = u32::to_le_bytes(usize_to_u32(last_offset, SPBFWriterWriteError::InvalidOffset.into())?);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x4; // NEXT

                out.write(&[0; 4])?; // NEXT
                out.write(&u16::to_le_bytes(usize_to_u16(data.data.len(), SPBFWriterWriteError::InvalidDataLength.into())?))?;
                out.write(&u16::to_le_bytes(data.data_id))?;
                let name = str_to_bytes_unaligned_small(data.name.clone());
                let name = if let Ok(ok) = name { ok } else { return Err(SPBFWriterWriteError::InvalidDataNameString.into()) };
                out.write(&name)?;
                out.write(&data.data)?;

                last_offset += 0x8 + name.len() + data.data.len();
            }
        }

        Ok(out)
    }
}
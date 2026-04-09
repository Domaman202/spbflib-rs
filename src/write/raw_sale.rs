use crate::raw::{bytes_align_small, str_to_bytes_align_small, usize_to_u16, usize_to_u32};
use crate::write::{SPBFWriter, SPBFWriterError, SPBFWriterWriteError};
use crate::{SPBFType, SPBFVersion};
use std::io::Write;

pub struct RawWriterSmallAlignedLittleEndian;

impl RawWriterSmallAlignedLittleEndian {
    pub fn write(writer: &mut SPBFWriter) -> Result<Vec<u8>, SPBFWriterError> {
        let mut out = Vec::new();

        out.write(b".SPBF\0")?;
        out.write(&u8::to_le_bytes(SPBFType::SmallAlignedLittleEndian.into()))?;
        out.write(&u8::to_le_bytes(SPBFVersion::LAST_SUPPORTED.into()))?;
        out.write(&[0; 4])?; // FORMAT_ENTRY
        out.write(&[0; 4])?; // DATA_ENTRY
        out.write(&u16::to_le_bytes(usize_to_u16(writer.build_name.len(), SPBFWriterWriteError::InvalidBuildNameLength.into())?))?;
        out.write(&u16::to_le_bytes(usize_to_u16(writer.build_version.len(), SPBFWriterWriteError::InvalidBuildVersionLength.into())?))?;
        let (name, name_align) = str_to_bytes_align_small(&writer.build_name);
        out.write(name)?;
        out.write(&vec![0; name_align])?;
        let (version, version_align) = str_to_bytes_align_small(&writer.build_version);
        out.write(version)?;
        out.write(&vec![0; version_align])?;

        let mut last_offset = 0x14 + name.len() + name_align + version.len() + version_align;

        if !writer.data_formats.is_empty() {
            let mut next_write_addr = 0x8..0xC; // FORMAT_ENTRY

            for format in &writer.data_formats {
                let block_start = u32::to_le_bytes(usize_to_u32(last_offset, SPBFWriterWriteError::InvalidOffset.into())?);
                out[next_write_addr].copy_from_slice(&block_start);
                next_write_addr = last_offset..last_offset + 0x4; // NEXT

                out.write(&[0; 4])?; // NEXT
                out.write(&u16::to_le_bytes(usize_to_u16(format.name.len(), SPBFWriterWriteError::InvalidDataFormatNameLength.into())?))?;
                out.write(&u16::to_le_bytes(format.data_id))?;
                let (name, name_align) = str_to_bytes_align_small(&format.name);
                out.write(name)?;
                out.write(&vec![0; name_align])?;

                last_offset += 0x8 + name.len() + name_align;
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
                out.write(&u16::to_le_bytes(usize_to_u16(data.name.len(), SPBFWriterWriteError::InvalidDataNameLength.into())?))?;
                let (name, name_align) = str_to_bytes_align_small(&data.name);
                out.write(name)?;
                out.write(&vec![0; name_align])?;
                let data_align = bytes_align_small(&data.data);
                out.write(&data.data)?;
                out.write(&vec![0; data_align])?;

                last_offset += 0xA + name.len() + name_align + data.data.len() + data_align;
            }
        }

        Ok(out)
    }
}
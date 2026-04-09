use crate::raw::align_len_small;
use crate::read::{SPBFDataForRead, SPBFDataFormatForRead, SPBFReadResult, SPBFReader, SPBFReaderDataReadError, SPBFReaderError, SPBFReaderDataFormatReadError, SPBFReaderHeaderReadError };
use crate::{SPBFType, SPBFVersion};

pub struct RawReaderSmallAlignedBigEndian;

impl RawReaderSmallAlignedBigEndian {
    pub fn read<'a>(reader: &'a SPBFReader, file_type: SPBFType, file_version: SPBFVersion) -> Result<SPBFReadResult<'a>, SPBFReaderError> {
        let source = reader.source;
        // Name
        let build_name_len = (&source[0x10..0x12]).try_into();
        let build_name_len = build_name_len.map_err(|_| SPBFReaderError::InvalidFileLength)?;
        let build_name_len = u16::from_be_bytes(build_name_len) as usize;
        let build_name_len_aligned = align_len_small(build_name_len);
        if source.len() < 0x14 + build_name_len_aligned { return Err(SPBFReaderHeaderReadError::InvalidBuildNameLength.into()) }
        let build_name = &source[0x14..0x14 + build_name_len];
        let build_name = Vec::from(build_name);
        let build_name = String::from_utf8(build_name);
        let build_name = build_name.map_err(|_| SPBFReaderHeaderReadError::InvalidBuildNameString.into())?;
        // Version
        let offset = build_name_len_aligned;
        let build_version_len = (&source[0x12..0x14]).try_into();
        let build_version_len = build_version_len.map_err(|_| SPBFReaderError::InvalidFileLength)?;
        let build_version_len = u16::from_be_bytes(build_version_len) as usize;
        if source.len() < offset + 0x14 + align_len_small(build_version_len) { return Err(SPBFReaderHeaderReadError::InvalidBuildVersionLength.into()) }
        let build_version = &source[offset + 0x14..offset + 0x14 + build_version_len];
        let build_version = Vec::from(build_version);
        let build_version = String::from_utf8(build_version);
        let build_version = build_version.map_err(|_| SPBFReaderHeaderReadError::InvalidBuildNameString.into())?;
        // Data Formats
        let offset = (&source[0x8..0xC]).try_into();
        let offset = offset.map_err(|_| SPBFReaderDataFormatReadError::InvalidOffset.into())?;
        let mut offset = u32::from_be_bytes(offset) as usize;
        let mut data_format_list = Vec::<SPBFDataFormatForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(SPBFReaderDataFormatReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x4]).try_into();
            let next_offset = next_offset.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let next_offset = u32::from_be_bytes(next_offset) as usize;
            let name_len = (&source[offset + 0x4..offset + 0x6]).try_into();
            let name_len = name_len.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let name_len = u16::from_be_bytes(name_len) as usize;
            let name_len_unaligned = name_len;
            let name_len = align_len_small(name_len);
            if source.len() < 0x8 + offset + name_len { return Err(SPBFReaderDataFormatReadError::InvalidNameLength.into()) }
            let data_id = (&source[offset + 0x6..offset + 0x8]).try_into();
            let data_id = data_id.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let data_id = u16::from_be_bytes(data_id);
            let name = &source[offset + 0x8..offset + 0x8 + name_len_unaligned];
            let name = Vec::from(name);
            let name = String::from_utf8(name);
            let name = name.map_err(|_| SPBFReaderDataFormatReadError::InvalidNameString.into())?;
            data_format_list.push(SPBFDataFormatForRead::new(data_id, name));
            offset = next_offset;
        }
        // Data
        let offset = (&source[0xC..0x10]).try_into();
        let offset = offset.map_err(|_| SPBFReaderDataReadError::InvalidOffset.into())?;
        let mut offset = u32::from_be_bytes(offset) as usize;
        let mut data_list = Vec::<SPBFDataForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(SPBFReaderDataReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x4]).try_into();
            let next_offset = next_offset.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let next_offset = u32::from_be_bytes(next_offset) as usize;
            let data_len = (&source[offset + 0x4..offset + 0x6]).try_into();
            let data_len = data_len.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let data_len = u16::from_be_bytes(data_len) as usize;
            let data_len_unaligned = data_len;
            let data_len = align_len_small(data_len);
            let data_id = (&source[offset + 0x6..offset + 0x8]).try_into();
            let data_id = data_id.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let data_id = u16::from_be_bytes(data_id);
            let format_position = data_format_list.iter().position(|it| it.data_id == data_id);
            let format_position = if let Some(some) = format_position { some } else { return Err(SPBFReaderDataReadError::InvalidDataId.into()) };
            let name_len = (&source[offset + 0x8..offset + 0xA]).try_into();
            let name_len = name_len.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let name_len = u16::from_be_bytes(name_len) as usize;
            let name_len_unaligned = name_len;
            let name_len = align_len_small(name_len);
            if source.len() < offset + 0xA + name_len { return Err(SPBFReaderDataReadError::InvalidNameLength.into()) }
            if source.len() < offset + 0xA + name_len + data_len { return Err(SPBFReaderDataReadError::InvalidDataLength.into()) }
            let name = &source[offset + 0xA..offset + 0xA + name_len_unaligned];
            let name = Vec::from(name);
            let name = String::from_utf8(name);
            let name = name.map_err(|_| SPBFReaderDataReadError::InvalidNameString.into())?;
            let data = &source[offset + 0xA + name_len..offset + 0xA + name_len + data_len_unaligned];
            data_list.push(SPBFDataForRead::new(name, data, data_id, format_position));
            offset = next_offset;
        }
        // Return
        Ok(
            SPBFReadResult {
                file_type,
                file_version,
                build_name,
                build_version,
                data_formats: data_format_list,
                data: data_list
            }
        )
    }
}
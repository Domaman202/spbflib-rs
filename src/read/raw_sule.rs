use crate::read::{SPBFDataForRead, SPBFDataFormatForRead, SPBFReadResult, SPBFReader, SPBFReaderDataReadError, SPBFReaderError, SPBFReaderFormatReadError, SPBFReaderHeaderReadError };
use crate::{SPBFType, SPBFVersion};
use std::ffi::CStr;

pub struct RawReaderSmallUnalignedLittleEndian;

impl RawReaderSmallUnalignedLittleEndian {
    pub fn read<'a>(reader: &'a SPBFReader, file_type: SPBFType, file_version: SPBFVersion) -> Result<SPBFReadResult<'a>, SPBFReaderError> {
        let source = reader.source;
        // Name
        let build_name_end = source[0x10..].iter().position(|&b| b == 0);
        let build_name_end = build_name_end.ok_or(SPBFReaderHeaderReadError::InvalidBuildNameLength.into())?;
        let build_name_end = build_name_end + 1;
        let build_name = &source[0x10..0x10 + build_name_end];
        let build_name = CStr::from_bytes_with_nul(build_name);
        let build_name = build_name.map_err(|_| SPBFReaderHeaderReadError::InvalidBuildNameString.into())?;
        let build_name = build_name.to_string_lossy().into_owned();
        // Version
        let offset = build_name_end;
        let build_version_end = source[offset + 0x10..].iter().position(|&b| b == 0);
        let build_version_end = build_version_end.ok_or(SPBFReaderHeaderReadError::InvalidBuildVersionLength.into())?;
        let build_version_end = build_version_end + 1;
        let build_version = &source[offset + 0x10.. offset + 0x10 + build_version_end];
        let build_version = CStr::from_bytes_with_nul(build_version);
        let build_version = build_version.map_err(|_| SPBFReaderHeaderReadError::InvalidBuildNameString.into())?;
        let build_version = build_version.to_string_lossy().into_owned();
        // Data Formats
        let offset = (&source[0x8..0xC]).try_into();
        let offset = offset.map_err(|_| SPBFReaderFormatReadError::InvalidOffset.into())?;
        let mut offset = u32::from_le_bytes(offset) as usize;
        let mut data_format_list = Vec::<SPBFDataFormatForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(SPBFReaderFormatReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x4]).try_into();
            let next_offset = next_offset.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let next_offset = u32::from_le_bytes(next_offset) as usize;
            let data_id = (&source[offset + 0x4..offset + 0x6]).try_into();
            let data_id = data_id.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let data_id = u16::from_le_bytes(data_id);
            let name_end = source[offset + 0x6..].iter().position(|&b| b == 0);
            let name_end = name_end.ok_or(SPBFReaderFormatReadError::InvalidNameString.into())?;
            let name_end = name_end + 1;
            let name = &source[offset + 0x6..offset + 0x6 + name_end];
            let name = CStr::from_bytes_with_nul(name);
            let name = name.map_err(|_| SPBFReaderFormatReadError::InvalidNameString.into())?;
            let name = name.to_string_lossy().into_owned();
            data_format_list.push(SPBFDataFormatForRead::new(data_id, name));
            offset = next_offset;
        }
        // Data
        let offset = (&source[0xC..0x10]).try_into();
        let offset = offset.map_err(|_| SPBFReaderDataReadError::InvalidOffset.into())?;
        let mut offset = u32::from_le_bytes(offset) as usize;
        let mut data_list = Vec::<SPBFDataForRead>::new();
        while offset != 0 {
            if source.len() < offset { return Err(SPBFReaderDataReadError::InvalidOffset.into()) }
            let next_offset = (&source[offset..offset + 0x4]).try_into();
            let next_offset = next_offset.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let next_offset = u32::from_le_bytes(next_offset) as usize;
            let data_len = (&source[offset + 0x4..offset + 0x6]).try_into();
            let data_len = data_len.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let data_len = u16::from_le_bytes(data_len) as usize;
            let data_id = (&source[offset + 0x6..offset + 0x8]).try_into();
            let data_id = data_id.map_err(|_| SPBFReaderError::InvalidFileLength)?;
            let data_id = u16::from_le_bytes(data_id);
            let format_position = data_format_list.iter().position(|it| it.data_id == data_id);
            let format_position = if let Some(some) = format_position { some } else { return Err(SPBFReaderDataReadError::InvalidDataId.into()) };
            if source.len() < offset + 0x8 + data_len { return Err(SPBFReaderDataReadError::InvalidDataLength.into()) }
            let name_end = source[offset + 0x8..].iter().position(|&b| b == 0);
            let name_end = name_end.ok_or(SPBFReaderDataReadError::InvalidNameString.into())?;
            let name_end = name_end + 1;
            let name = &source[offset + 0x8..offset + 0x8 + name_end];
            let name = CStr::from_bytes_with_nul(name);
            let name = name.map_err(|_| SPBFReaderDataReadError::InvalidNameString.into())?;
            let name_len = name.to_bytes_with_nul().len();
            let name = name.to_string_lossy().into_owned();
            let data = &source[offset + 0x8 + name_len..offset + 0x8 + name_len + data_len];
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
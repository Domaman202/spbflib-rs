use crate::read::raw::{check_header_len, check_magic, read_type, read_version};
use crate::read::raw_sabe::RawReaderSmallAlignedBigEndian;
use crate::read::raw_sale::RawReaderSmallAlignedLittleEndian;
use crate::read::raw_sube::RawReaderSmallUnalignedBigEndian;
use crate::read::raw_sule::RawReaderSmallUnalignedLittleEndian;
use crate::{SPBFType, SPBFVersion};

pub mod raw;
mod raw_sabe;
mod raw_sale;
mod raw_sube;
mod raw_sule;

#[derive(Debug)]
pub struct SPBFReader<'a> {
    source: &'a [u8],
    file_type: SPBFType,
    file_version: SPBFVersion
}

#[derive(Debug, Clone)]
pub struct SPBFReadResult<'a> {
    file_type: SPBFType,
    file_version: SPBFVersion,
    build_name: String,
    build_version: String,
    data_formats: Vec<SPBFDataFormatForRead>,
    data: Vec<SPBFDataForRead<'a>>
}

#[derive(Debug, Clone)]
pub struct SPBFDataFormatForRead {
    data_id: u16,
    name: String
}

#[derive(Debug, Clone)]
pub struct SPBFDataForRead<'a> {
    name: String,
    data: &'a [u8],
    data_id: u16,
    format_position: usize,
}

#[derive(Debug)]
pub enum SPBFReaderError {
    InvalidFileLength,
    Header(SPBFReaderHeaderReadError),
    DataFormat(SPBFReaderFormatReadError),
    Data(SPBFReaderDataReadError)
}

#[derive(Debug)]
pub enum SPBFReaderHeaderReadError {
    InvalidMagic,
    InvalidType,
    UnsupportedVersion,
    InvalidBuildNameLength,
    InvalidBuildNameString,
    InvalidBuildVersionLength,
    InvalidBuildVersionString,
}

#[derive(Debug)]
pub enum SPBFReaderFormatReadError {
    InvalidOffset,
    InvalidNameLength,
    InvalidNameString,
}

#[derive(Debug)]
pub enum SPBFReaderDataReadError {
    InvalidOffset,
    InvalidNameLength,
    InvalidNameString,
    InvalidDataLength,
    InvalidDataId
}

impl<'a> SPBFReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, SPBFReaderError> {
        if !check_header_len(bytes) { return Err(SPBFReaderError::InvalidFileLength) }
        if !check_magic(bytes) { return Err(SPBFReaderError::Header(SPBFReaderHeaderReadError::InvalidMagic)) }
        let file_type = read_type(bytes).map_err(|()| SPBFReaderHeaderReadError::InvalidType.into())?;
        let file_version = read_version(bytes);
        Ok(
            Self {
                source: bytes,
                file_type,
                file_version
            }
        )
    }

    pub fn file_type(&self) -> SPBFType {
        self.file_type
    }

    pub fn file_version(&self) -> SPBFVersion {
        self.file_version
    }

    pub fn is_read_supported(&self) -> bool {
        self.file_version.is_supported()
    }

    pub fn read(&'_ self) -> Result<SPBFReadResult<'_>, SPBFReaderError> {
        if !self.is_read_supported() { return Err(SPBFReaderHeaderReadError::UnsupportedVersion.into()) };
        match self.file_type {
            SPBFType::SmallUnalignedLittleEndian => RawReaderSmallUnalignedLittleEndian::read(&self, self.file_type, self.file_version),
            SPBFType::SmallUnalignedBigEndian    => RawReaderSmallUnalignedBigEndian::read(&self, self.file_type, self.file_version),
            SPBFType::SmallAlignedLittleEndian   => RawReaderSmallAlignedLittleEndian::read(&self, self.file_type, self.file_version),
            SPBFType::SmallAlignedBigEndian      => RawReaderSmallAlignedBigEndian::read(&self, self.file_type, self.file_version)
        }
    }
}

impl<'a> SPBFReadResult<'a> {
    pub fn file_type(&self) -> SPBFType {
        self.file_type
    }

    pub fn file_version(&self) -> SPBFVersion {
        self.file_version
    }

    pub fn build_name(&self) -> &String {
        &self.build_name
    }

    pub fn build_version(&self) -> &String {
        &self.build_version
    }

    pub fn data_formats(&self) -> &Vec<SPBFDataFormatForRead> {
        &self.data_formats
    }

    pub fn data(&self) -> &Vec<SPBFDataForRead<'a>> {
        &self.data
    }
}

impl SPBFDataFormatForRead {
    pub fn new(data_id: u16, name: String) -> Self {
        Self { data_id, name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data_id(&self) -> u16 {
        self.data_id
    }
}

impl<'a> SPBFDataForRead<'a> {
    pub fn new(name: String, data: &'a [u8], data_id: u16, format_position: usize) -> Self {
        Self { name, data, data_id, format_position }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }
    
    pub fn data_id(&self) -> u16 {
        self.data_id
    }

    pub fn format<'b>(&self, read: &'b SPBFReadResult) -> &'b SPBFDataFormatForRead {
        unsafe { &read.data_formats.get_unchecked(self.format_position) }
    }
}

impl Into<SPBFReaderError> for SPBFReaderHeaderReadError {
    fn into(self) -> SPBFReaderError {
        SPBFReaderError::Header(self)
    }
}

impl Into<SPBFReaderError> for SPBFReaderFormatReadError {
    fn into(self) -> SPBFReaderError {
        SPBFReaderError::DataFormat(self)
    }
}

impl Into<SPBFReaderError> for SPBFReaderDataReadError {
    fn into(self) -> SPBFReaderError {
        SPBFReaderError::Data(self)
    }
}
use crate::read::{SPBFReadResult, SPBFReaderError};
use crate::write::raw_sabe::RawWriterSmallAlignedBigEndian;
use crate::write::raw_sale::RawWriterSmallAlignedLittleEndian;
use crate::write::raw_sube::RawWriterSmallUnalignedBigEndian;
use crate::write::raw_sule::RawWriterSmallUnalignedLittleEndian;
use crate::{SPBFType, SPBFVersion};

mod raw_sabe;
mod raw_sale;
mod raw_sube;
mod raw_sule;

#[derive(Debug)]
pub struct SPBFWriter {
    pub build_name: String,
    pub build_version: String,
    data_format_id_last: u16,
    data_format_id_pool: Vec<u16>,
    data_formats: Vec<SPBFDataFormatForWrite>,
    data: Vec<SPBFDataForWrite>
}

#[derive(Debug, Clone)]
pub struct SPBFDataFormatForWrite {
    data_id: u16,
    name: String,
    refs: u32
}

#[derive(Debug, Clone)]
pub struct SPBFDataForWrite {
    data_id: u16,
    name: String,
    data: Box<[u8]>
}

#[derive(Debug)]
pub enum SPBFWriterError {
    DataAdd(SPBFWriterDataAddError),
    Write(SPBFWriterWriteError)
}

#[derive(Debug)]
pub enum SPBFWriterDataAddError {
    DataAlreadyDefined,
    FormatCounterOverflow
}

#[derive(Debug)]
pub enum SPBFWriterWriteError {
    UnsupportedVersion,
    InvalidBuildNameLength,
    InvalidBuildNameString,
    InvalidBuildVersionLength,
    InvalidBuildVersionString,
    InvalidDataFormatNameLength,
    InvalidDataFormatNameString,
    InvalidDataNameLength,
    InvalidDataNameString,
    InvalidDataLength,
    InvalidOffset,
    IOError(std::io::Error),
}

impl SPBFWriter {
    pub fn new(build_name: String, build_version: String) -> Self {
        Self {
            build_name,
            build_version,
            data_format_id_last: 0xFF, // 0xFF - last reserved
            data_format_id_pool: Vec::new(),
            data_formats: Vec::new(),
            data: Vec::new(),
        }
    }

    fn find_or_add_format(&mut self, name: &String) -> Result<u16, SPBFWriterError> {
        let format = self.data_formats.iter_mut().find(|it| it.name == *name);
        if let Some(format) = format {
            format.refs += 1;
            return Ok(format.data_id);
        }

        if let Some(id) = self.data_format_id_pool.pop() {
            self.data_formats.push(SPBFDataFormatForWrite::new(id, name.clone(), 1));
            return Ok(id);
        }

        if self.data_format_id_last == u16::MIN {
            return Err(SPBFWriterDataAddError::FormatCounterOverflow.into());
        }

        self.data_format_id_last += 1;
        let id = self.data_format_id_last;
        self.data_formats.push(SPBFDataFormatForWrite::new(id, name.clone(), 1));
        Ok(id)
    }

    fn find_or_add_format_unchecked(&mut self, name: &String) -> u16 {
        let format = self.data_formats.iter_mut().find(|it| it.name == *name);
        if let Some(format) = format {
            format.refs += 1;
            return format.data_id;
        }

        if let Some(id) = self.data_format_id_pool.pop() {
            self.data_formats.push(SPBFDataFormatForWrite::new(id, name.clone(), 1));
            return id;
        }

        self.data_format_id_last += 1;
        let id = self.data_format_id_last;
        self.data_formats.push(SPBFDataFormatForWrite::new(id, name.clone(), 1));
        id
    }

    pub fn data_formats(&self) -> &Vec<SPBFDataFormatForWrite> {
        &self.data_formats
    }

    pub fn add_data(&mut self, name: String, format: &String, bytes: Box<[u8]>) -> Result<(), SPBFWriterError> {
        if let Some(_) = self.data.iter().find(|it| it.name == *name) {
            Err(SPBFWriterDataAddError::DataAlreadyDefined.into())
        } else {
            let data_id = self.find_or_add_format(format)?;
            self.data.push(SPBFDataForWrite::new(data_id, name, bytes));
            Ok(())
        }
    }

    pub unsafe fn add_data_unchecked(&mut self, name: String, format: &String, bytes: Box<[u8]>) {
        let data_id = self.find_or_add_format_unchecked(format);
        self.data.push(SPBFDataForWrite::new(data_id, name, bytes));
    }

    pub fn add_or_overwrite_data(&mut self, name: &String, format: &String, bytes: Box<[u8]>) -> Result<(), SPBFWriterError> {
        let data_id = self.find_or_add_format(format)?;
        if let Some(data) = self.data.iter_mut().find(|it| it.name == *name) {
            data.data_id = data_id;
            data.data = bytes;
            Ok(())
        } else {
            self.data.push(SPBFDataForWrite::new(data_id, name.clone(), bytes));
            Ok(())
        }
    }

    pub fn remove_data(&mut self, name: &String) -> bool {
        if let Some(idx) = self.data.iter().position(|it| it.name == *name) {
            let data = self.data.remove(idx);
            let format_idx = self.data_formats.iter().position(|it| it.data_id == data.data_id);
            let format_idx = unsafe { format_idx.unwrap_unchecked() };
            let format = &mut self.data_formats[format_idx];
            format.refs -= 1;
            if format.refs == 0 {
                self.data_format_id_pool.push(format.data_id);
                self.data_formats.remove(format_idx);
            }
            true
        } else {
            false
        }
    }

    pub fn data(&self) -> &Vec<SPBFDataForWrite> {
        &self.data
    }

    pub fn write(&mut self, r#type: SPBFType, version: SPBFVersion) -> Result<Vec<u8>, SPBFWriterError> {
        if !version.is_supported() { return Err(SPBFWriterWriteError::UnsupportedVersion.into()); }
        match r#type {
            SPBFType::SmallUnalignedLittleEndian => RawWriterSmallUnalignedLittleEndian::write(self),
            SPBFType::SmallUnalignedBigEndian    => RawWriterSmallUnalignedBigEndian::write(self),
            SPBFType::SmallAlignedLittleEndian   => RawWriterSmallAlignedLittleEndian::write(self),
            SPBFType::SmallAlignedBigEndian      => RawWriterSmallAlignedBigEndian::write(self)
        }
    }
}

impl TryFrom<&SPBFReadResult<'_>> for SPBFWriter {
    type Error = SPBFReaderError;

    fn try_from(value: &SPBFReadResult) -> Result<Self, SPBFReaderError> {
        let mut data_format_id_last = 0xFF; // 0xFF - last reserved
        let mut data_formats: Vec<SPBFDataFormatForWrite> =
            value
                .data_formats()
                .iter()
                .map(|it| {
                    let id = it.data_id();
                    if data_format_id_last < id { data_format_id_last = id }
                    SPBFDataFormatForWrite::new(id, it.name().clone(), 0)
                })
                .collect();
        let data: Vec<SPBFDataForWrite> =
            value
                .data()
                .iter()
                .map(|it| {
                    let id = it.data_id();
                    let format = unsafe { data_formats.iter_mut().find(|it| it.data_id == id).unwrap_unchecked() };
                    format.refs += 1;
                    SPBFDataForWrite::new(id, it.name().clone(), it.data().into())
                })
                .collect();
        Ok(
            Self {
                build_name: value.build_name().clone(),
                build_version: value.build_version().clone(),
                data_format_id_last,
                data_format_id_pool: Vec::new(),
                data_formats,
                data
            }
        )
    }
}

impl SPBFDataFormatForWrite {
    pub fn new(data_id: u16, name: String, refs: u32) -> Self {
        Self { data_id, name, refs }
    }

    pub fn data_id(&self) -> u16 {
        self.data_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn refs(&self) -> u32 {
        self.refs
    }
}

impl SPBFDataForWrite {
    pub fn new(data_id: u16, name: String, data: Box<[u8]>) -> Self {
        Self { data_id, name, data }
    }

    pub fn data_id(&self) -> u16 {
        self.data_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl From<std::io::Error> for SPBFWriterError {
    fn from(value: std::io::Error) -> Self {
        Self::Write(SPBFWriterWriteError::IOError(value))
    }
}

impl Into<SPBFWriterError> for SPBFWriterDataAddError {
    fn into(self) -> SPBFWriterError {
        SPBFWriterError::DataAdd(self)
    }
}

impl Into<SPBFWriterError> for SPBFWriterWriteError {
    fn into(self) -> SPBFWriterError {
        SPBFWriterError::Write(self)
    }
}

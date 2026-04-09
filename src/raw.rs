use crate::write::SPBFWriterError;
use std::ffi::CString;

pub fn align_len_small(len: usize) -> usize {
    len + (len & 1)
}

pub fn usize_to_u32(value: usize, err: SPBFWriterError) -> Result<u32, SPBFWriterError> {
    if value >= u32::MAX as usize {
        Err(err)
    } else {
        Ok(value as u32)
    }
}

pub fn usize_to_u16(value: usize, err: SPBFWriterError) -> Result<u16, SPBFWriterError> {
    if value >= u16::MAX as usize {
        Err(err)
    } else {
        Ok(value as u16)
    }
}

pub fn bytes_align_small(bytes: &[u8]) -> usize {
    let len = bytes.len();
    align_len_small(len) - len
}

pub fn str_to_bytes_align_small(str: &String) -> (&[u8], usize) {
    let len = str.len();
    let align = align_len_small(len);
    (str.as_bytes(), align - len)
}

pub fn str_to_bytes_unaligned_small(str: String) -> Result<Vec<u8>, ()> {
    match CString::new(str) {
        Ok(ok) => Ok(ok.into_bytes_with_nul()),
        Err(_) => Err(())
    }
}
pub mod raw;
pub mod read;
pub mod write;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SPBFType {
    SmallUnalignedLittleEndian = 0x0,
    SmallUnalignedBigEndian    = 0x1,
    SmallAlignedLittleEndian   = 0x2,
    SmallAlignedBigEndian      = 0x3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SPBFVersion(u8);

impl TryFrom<u8> for SPBFType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 0x3 { return Err(()); }
        unsafe { std::mem::transmute(value) }
    }
}

impl Into<u8> for SPBFType {
    fn into(self) -> u8 {
        unsafe {  std::mem::transmute(self) }
    }
}

impl SPBFVersion {
    pub const V0: Self = Self(0x0);
    pub const V1: Self = Self(0x1);
    pub const LAST_SUPPORTED: Self = Self::V1;

    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn is_supported(self) -> bool {
        self.0 == Self::LAST_SUPPORTED.0
    }

    pub const fn as_raw(self) -> u8 {
        self.0
    }
}

impl Into<u8> for SPBFVersion {
    fn into(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read::{SPBFReader, SPBFReaderError, SPBFReaderDataFormatReadError, SPBFReaderHeaderReadError };
    use write::{SPBFWriter, SPBFWriterDataAddError, SPBFWriterError, SPBFWriterWriteError };

    // Helper: create a writer with some data formats and data blocks
    fn create_test_writer(
        build_name: &str,
        build_version: &str,
        data_specs: &[(&str, &str, &[u8])], // (data_name, format_name, data_bytes)
    ) -> SPBFWriter {
        let mut writer = SPBFWriter::new(build_name.to_string(), build_version.to_string());
        for (data_name, format_name, data_bytes) in data_specs {
            writer
                .add_data(
                    data_name.to_string(),
                    &format_name.to_string(),
                    data_bytes.to_vec().into_boxed_slice(),
                )
                .unwrap();
        }
        writer
    }

    #[test]
    fn roundtrip_all_types() {
        let specs = &[
            ("data1", "formatA", &b"hello"[..]),
            ("data2", "formatB", &b"world"[..]),
            ("data3", "formatA", &b"again"[..]),
        ];
        let build_name = "@test_build";
        let build_version = "1.2.3";

        for ty in [
            SPBFType::SmallUnalignedLittleEndian,
            SPBFType::SmallUnalignedBigEndian,
            SPBFType::SmallAlignedLittleEndian,
            SPBFType::SmallAlignedBigEndian,
        ] {
            let mut writer = create_test_writer(build_name, build_version, specs);
            let bytes = writer.write(ty, SPBFVersion::LAST_SUPPORTED).unwrap();
            let reader = SPBFReader::new(&bytes).unwrap();
            let read_result = reader.read().unwrap();

            assert_eq!(read_result.file_type(), ty);
            assert_eq!(read_result.build_name(), build_name);
            assert_eq!(read_result.build_version(), build_version);

            let formats = read_result.data_formats();
            assert_eq!(formats.len(), 2);
            let format_a = formats.iter().find(|f| f.name() == "formatA").unwrap();
            let format_b = formats.iter().find(|f| f.name() == "formatB").unwrap();
            let data = read_result.data();
            assert_eq!(data.len(), 3);
            let data1 = data.iter().find(|d| d.name() == "data1").unwrap();
            let data2 = data.iter().find(|d| d.name() == "data2").unwrap();
            let data3 = data.iter().find(|d| d.name() == "data3").unwrap();
            assert_eq!(data1.data_id(), format_a.data_id());
            assert_eq!(data3.data_id(), format_a.data_id());
            assert_eq!(data2.data_id(), format_b.data_id());
            assert_eq!(data1.data(), b"hello");
            assert_eq!(data2.data(), b"world");
            assert_eq!(data3.data(), b"again");
        }
    }

    #[test]
    fn data_format_reuse() {
        let mut writer = SPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("d1".to_string(), &"fmt".to_string(), b"data1".to_vec().into_boxed_slice())
            .unwrap();
        writer
            .add_data("d2".to_string(), &"fmt".to_string(), b"data2".to_vec().into_boxed_slice())
            .unwrap();

        let formats = writer.data_formats();
        assert_eq!(formats.len(), 1);
        assert_eq!(formats[0].name(), "fmt");
        assert_eq!(formats[0].refs(), 2);
    }

    #[test]
    fn remove_data_decrements_refs() {
        let mut writer = SPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("d1".to_string(), &"fmt".to_string(), b"d1".to_vec().into_boxed_slice())
            .unwrap();
        writer
            .add_data("d2".to_string(), &"fmt".to_string(), b"d2".to_vec().into_boxed_slice())
            .unwrap();

        assert_eq!(writer.data_formats().len(), 1);
        assert_eq!(writer.data_formats()[0].refs(), 2);

        assert!(writer.remove_data(&"d1".to_string()));
        assert_eq!(writer.data().len(), 1);
        assert_eq!(writer.data_formats().len(), 1);
        assert_eq!(writer.data_formats()[0].refs(), 1);

        assert!(writer.remove_data(&"d2".to_string()));
        assert_eq!(writer.data().len(), 0);
        assert_eq!(writer.data_formats().len(), 0);

        writer
            .add_data("d3".to_string(), &"fmt".to_string(), b"d3".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data_formats().len(), 1);
    }

    #[test]
    fn add_data_duplicate_name_error() {
        let mut writer = SPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("dup".to_string(), &"fmt".to_string(), b"first".to_vec().into_boxed_slice())
            .unwrap();
        let err = writer
            .add_data("dup".to_string(), &"fmt".to_string(), b"second".to_vec().into_boxed_slice())
            .unwrap_err();
        match err {
            SPBFWriterError::DataAdd(SPBFWriterDataAddError::DataAlreadyDefined) => (),
            _ => panic!("expected DataAlreadyDefined"),
        }
    }

    #[test]
    fn add_or_overwrite_data() {
        let mut writer = SPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_or_overwrite_data(&"d1".to_string(), &"fmt".to_string(), b"first".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data().len(), 1);
        assert_eq!(writer.data()[0].data(), b"first");

        writer
            .add_or_overwrite_data(&"d1".to_string(), &"fmt2".to_string(), b"second".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data().len(), 1);
        assert_eq!(writer.data()[0].data(), b"second");
        let fmt = writer.data_formats().iter().find(|f| f.data_id() == writer.data()[0].data_id()).unwrap();
        assert_eq!(fmt.name(), "fmt2");
    }

    #[test]
    fn remove_nonexistent_data_returns_false() {
        let mut writer = SPBFWriter::new("test".to_string(), "1.0".to_string());
        assert!(!writer.remove_data(&"nosuch".to_string()));
    }

    #[test]
    fn read_invalid_magic() {
        let bytes = b"BADMAGIC";
        let err = SPBFReader::new(bytes).unwrap_err();
        match err {
            SPBFReaderError::Header(SPBFReaderHeaderReadError::InvalidMagic) => (),
            _ => panic!("expected InvalidMagic"),
        }
    }

    #[test]
    fn read_unsupported_version() {
        let mut bytes = b".SPBF\0".to_vec();
        bytes.push(0x00); // type
        bytes.push(0xFF); // unsupported version
        bytes.resize(8, 0);
        let reader = SPBFReader::new(&bytes).unwrap();
        assert_eq!(reader.is_read_supported(), false);
        let err = reader.read().unwrap_err();
        match err {
            SPBFReaderError::Header(SPBFReaderHeaderReadError::UnsupportedVersion) => (),
            _ => panic!("expected UnsupportedVersion"),
        }
    }

    #[test]
    fn read_truncated_file() {
        let bytes = b".SPBF\0";
        let err = SPBFReader::new(bytes).unwrap_err();
        match err {
            SPBFReaderError::InvalidFileLength => (),
            _ => panic!("expected InvalidFileLength"),
        }
    }

    #[test]
    fn read_truncated_name() {
        let mut writer = SPBFWriter::new("very_long_build_name_that_exceeds_buffer".to_string(), "1.0".to_string());
        let mut bytes = writer
            .write(SPBFType::SmallAlignedLittleEndian, SPBFVersion::LAST_SUPPORTED)
            .unwrap();
        // Truncate after the header but before the full name (0x20 is safe for aligned case)
        bytes.truncate(0x20);
        let reader = SPBFReader::new(&bytes).unwrap();
        let err = reader.read().unwrap_err();
        match err {
            SPBFReaderError::Header(SPBFReaderHeaderReadError::InvalidBuildNameLength) => (),
            _ => panic!("expected InvalidLength"),
        }
    }

    #[test]
    fn data_format_lookup() {
        let mut writer = create_test_writer(
            "test",
            "1.0",
            &[("data1", "formatX", &b"abc"[..]), ("data2", "formatY", &b"def"[..])],
        );
        let bytes = writer.write(SPBFType::SmallAlignedLittleEndian, SPBFVersion::LAST_SUPPORTED).unwrap();
        let reader = SPBFReader::new(&bytes).unwrap();
        let read_result = reader.read().unwrap();

        let data = read_result.data();
        let data1 = data.iter().find(|d| d.name() == "data1").unwrap();
        let fmt1 = data1.format(&read_result);
        assert_eq!(fmt1.name(), "formatX");
        let data2 = data.iter().find(|d| d.name() == "data2").unwrap();
        let fmt2 = data2.format(&read_result);
        assert_eq!(fmt2.name(), "formatY");
    }

    #[test]
    fn empty_writer() {
        let mut writer = SPBFWriter::new("empty".to_string(), "0.0".to_string());
        let bytes = writer.write(SPBFType::SmallAlignedLittleEndian, SPBFVersion::LAST_SUPPORTED).unwrap();
        let reader = SPBFReader::new(&bytes).unwrap();
        let read_result = reader.read().unwrap();

        assert_eq!(read_result.data_formats().len(), 0);
        assert_eq!(read_result.data().len(), 0);
        assert_eq!(read_result.build_name(), "empty");
        assert_eq!(read_result.build_version(), "0.0");
    }

    #[test]
    fn convert_from_read_result() {
        let mut writer_orig = create_test_writer(
            "convert",
            "2.0",
            &[("c1", "fmtA", &b"one"[..]), ("c2", "fmtB", &b"two"[..]), ("c3", "fmtA", &b"three"[..])],
        );
        let bytes = writer_orig.write(SPBFType::SmallAlignedBigEndian, SPBFVersion::LAST_SUPPORTED).unwrap();
        let reader = SPBFReader::new(&bytes).unwrap();
        let read_result = reader.read().unwrap();

        let writer_converted: SPBFWriter = SPBFWriter::try_from(&read_result).unwrap();
        let mut writer_converted_mut = writer_converted;
        let bytes2 = writer_converted_mut.write(SPBFType::SmallAlignedBigEndian, SPBFVersion::LAST_SUPPORTED).unwrap();
        // The two byte sequences should be identical (deterministic writer)
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn data_id_reuse_after_remove() {
        let mut writer = SPBFWriter::new("test".to_string(), "1.0".to_string());
        writer
            .add_data("d1".to_string(), &"fmt".to_string(), b"".to_vec().into_boxed_slice())
            .unwrap();
        let first_format_id = writer.data_formats()[0].data_id();
        writer.remove_data(&"d1".to_string());
        writer
            .add_data("d2".to_string(), &"fmt".to_string(), b"".to_vec().into_boxed_slice())
            .unwrap();
        assert_eq!(writer.data_formats()[0].data_id(), first_format_id);
    }

    #[test]
    fn writer_errors_on_too_long_strings() {
        let mut writer = SPBFWriter::new("a".repeat(u16::MAX as usize).to_string(), "1.0".to_string());
        let err = writer
            .write(SPBFType::SmallAlignedLittleEndian, SPBFVersion::LAST_SUPPORTED)
            .unwrap_err();
        match err {
            SPBFWriterError::Write(SPBFWriterWriteError::InvalidNameLength) => (),
            _ => panic!("expected InvalidNameLength"),
        }
    }

    #[test]
    fn read_invalid_type_byte() {
        let mut bytes = b".SPBF\0".to_vec();
        bytes.push(0xFF); // invalid type
        bytes.push(SPBFVersion::LAST_SUPPORTED.into());
        bytes.resize(8, 0);
        let err = SPBFReader::new(&bytes).unwrap_err();
        match err {
            SPBFReaderError::Header(SPBFReaderHeaderReadError::InvalidType) => (),
            _ => panic!("expected InvalidType"),
        }
    }

    #[test]
    fn read_corrupted_format_next_offset() {
        let mut writer = create_test_writer("corrupt", "1.0", &[("d1", "fmt", &b"x"[..])]);
        let mut bytes = writer
            .write(SPBFType::SmallAlignedLittleEndian, SPBFVersion::LAST_SUPPORTED)
            .unwrap();
        // For SPBF aligned, FORMAT_ENTRY is at offset 0x8 (u32)
        let format_offset = u32::from_le_bytes(bytes[0x8..0xC].try_into().unwrap()) as usize;
        let next_offset = format_offset + 0x10000;
        bytes[format_offset..format_offset + 4].copy_from_slice(&(next_offset as u32).to_le_bytes());
        let reader = SPBFReader::new(&bytes).unwrap();
        let err = reader.read().unwrap_err();
        match err {
            SPBFReaderError::DataFormat(SPBFReaderDataFormatReadError::InvalidOffset) => (),
            _ => panic!("expected InvalidOffset"),
        }
    }

    #[test]
    fn debug_manual_read_unaligned() {
        let build_name = "@test_build";
        let build_version = "1.2.3";
        let specs = &[("data1", "formatA", &b"hello"[..])];

        for ty in [
            SPBFType::SmallUnalignedLittleEndian,
            SPBFType::SmallUnalignedBigEndian,
        ] {
            let mut writer = create_test_writer(build_name, build_version, specs);
            let bytes = writer.write(ty, SPBFVersion::LAST_SUPPORTED).unwrap();

            println!("\n=== Manual read for {:?} ===", ty);
            println!("Total bytes: {}", bytes.len());

            // 1. Проверяем, что CStr::from_bytes_with_nul работает напрямую
            let name_slice = &bytes[0x10..];
            let name_cstr = match std::ffi::CStr::from_bytes_with_nul(name_slice) {
                Ok(cstr) => {
                    println!("✅ Name CStr: {:?}", cstr);
                    cstr
                }
                Err(e) => {
                    println!("❌ Name CStr error: {:?}", e);
                    continue;
                }
            };
            let name_len_with_nul = name_cstr.to_bytes_with_nul().len();
            println!("Name length with null: {}", name_len_with_nul);

            // 2. Читаем версию
            let version_start = 0x10 + name_len_with_nul;
            let version_slice = &bytes[version_start..];
            let version_cstr = match std::ffi::CStr::from_bytes_with_nul(version_slice) {
                Ok(cstr) => {
                    println!("✅ Version CStr: {:?}", cstr);
                    cstr
                }
                Err(e) => {
                    println!("❌ Version CStr error: {:?}", e);
                    continue;
                }
            };
            let version_len_with_nul = version_cstr.to_bytes_with_nul().len();
            println!("Version length with null: {}", version_len_with_nul);

            // 3. Проверяем, что после версии есть хотя бы один байт (для данных)
            let after_version = version_start + version_len_with_nul;
            println!("First byte after version: 0x{:02x}", bytes.get(after_version).unwrap_or(&0xFF));

            // 4. Пробуем создать reader и вызвать read (для сравнения)
            let reader = match SPBFReader::new(&bytes) {
                Ok(r) => r,
                Err(e) => {
                    println!("Reader creation error: {:?}", e);
                    continue;
                }
            };
            match reader.read() {
                Ok(result) => println!("✅ Full read OK: name={}, version={}", result.build_name(), result.build_version()),
                Err(e) => println!("❌ Full read error: {:?}", e),
            }
        }
    }
}
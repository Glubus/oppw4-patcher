use std::{
    fmt,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

pub trait ReadSeek: Read + Seek {}

impl<T> ReadSeek for T where T: Read + Seek {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReplacementSource {
    File(PathBuf),
    Memory {
        name: String,
        bytes: Vec<u8>,
    },
    FileRange {
        path: PathBuf,
        offset: u64,
        size: u64,
    },
    ZipEntry {
        zip_path: PathBuf,
        entry_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModAsset {
    pub file_name: String,
    pub source: ReplacementSource,
}

impl ReplacementSource {
    pub fn display_name(&self) -> String {
        match self {
            Self::File(path) => path.display().to_string(),
            Self::Memory { name, bytes } => format!("{name}@memory:0x{:x}", bytes.len()),
            Self::FileRange { path, offset, size } => {
                format!("{}@0x{offset:x}+0x{size:x}", path.display())
            }
            Self::ZipEntry {
                zip_path,
                entry_name,
            } => format!("{}!{}", zip_path.display(), entry_name),
        }
    }

    pub fn backing_path(&self) -> &Path {
        match self {
            Self::File(path) => path,
            Self::Memory { .. } => Path::new("<memory>"),
            Self::FileRange { path, .. } => path,
            Self::ZipEntry { zip_path, .. } => zip_path,
        }
    }

    pub fn payload_size(&self) -> std::io::Result<u64> {
        match self {
            Self::File(path) => std::fs::metadata(path).map(|metadata| metadata.len()),
            Self::Memory { bytes, .. } => Ok(bytes.len() as u64),
            Self::FileRange { size, .. } => Ok(*size),
            Self::ZipEntry {
                zip_path,
                entry_name,
            } => {
                let file = File::open(zip_path)?;
                let mut archive = open_zip(file)?;
                let entry = archive.by_name(entry_name).map_err(zip_error)?;
                Ok(entry.size())
            }
        }
    }

    pub fn modified_time(&self) -> Option<SystemTime> {
        if matches!(self, Self::Memory { .. }) {
            return None;
        }
        std::fs::metadata(self.backing_path())
            .ok()
            .and_then(|metadata| metadata.modified().ok())
    }

    pub fn open_reader(&self) -> std::io::Result<Box<dyn ReadSeek + Send>> {
        match self {
            Self::File(path) => Ok(Box::new(File::open(path)?)),
            Self::Memory { bytes, .. } => Ok(Box::new(Cursor::new(bytes.clone()))),
            Self::FileRange { path, offset, size } => Ok(Box::new(Cursor::new(read_file_range(
                path, *offset, *size,
            )?))),
            Self::ZipEntry {
                zip_path,
                entry_name,
            } => Ok(Box::new(Cursor::new(read_zip_entry(zip_path, entry_name)?))),
        }
    }

    pub fn read_range(&self, offset: u64, target: &mut [u8]) -> std::io::Result<()> {
        let mut reader = self.open_reader()?;
        reader.seek(SeekFrom::Start(offset))?;
        reader.read_exact(target)
    }

    pub fn memory_ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        match self {
            Self::File(_) => {}
            Self::Memory { name, bytes } => {
                push_memory_range(&mut ranges, name.as_ptr() as usize, name.len());
                push_memory_range(&mut ranges, bytes.as_ptr() as usize, bytes.len());
            }
            Self::FileRange { .. } => {}
            Self::ZipEntry { entry_name, .. } => {
                push_memory_range(&mut ranges, entry_name.as_ptr() as usize, entry_name.len());
            }
        }
        ranges
    }
}

impl fmt::Display for ReplacementSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_name())
    }
}

fn read_file_range(path: &Path, offset: u64, size: u64) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let capacity = size.min(usize::MAX as u64) as usize;
    let mut bytes = vec![0u8; capacity];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_zip_entry(zip_path: &Path, entry_name: &str) -> std::io::Result<Vec<u8>> {
    let file = File::open(zip_path)?;
    let mut archive = open_zip(file)?;
    let mut entry = archive.by_name(entry_name).map_err(zip_error)?;
    let capacity = entry.size().min(usize::MAX as u64) as usize;
    let mut bytes = Vec::with_capacity(capacity);
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn open_zip(file: File) -> std::io::Result<zip::ZipArchive<File>> {
    zip::ZipArchive::new(file).map_err(zip_error)
}

fn zip_error(error: zip::result::ZipError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

fn push_memory_range(ranges: &mut Vec<(usize, usize)>, start: usize, len: usize) {
    if start == 0 || len == 0 {
        return;
    }
    if let Some(end) = start.checked_add(len) {
        ranges.push((start, end));
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn zip_entry_source_reports_size_and_reads_ranges() {
        let root = temp_root("zip-source");
        let zip_path = root.join("mod.zip");
        write_zip(&zip_path, "ScreenLayout/portrait.g1t", b"abcdef");
        let source = ReplacementSource::ZipEntry {
            zip_path: zip_path.clone(),
            entry_name: "ScreenLayout/portrait.g1t".to_string(),
        };
        let mut range = [0u8; 3];

        assert_eq!(source.payload_size().unwrap(), 6);
        source.read_range(2, &mut range).unwrap();
        assert_eq!(&range, b"cde");
        assert_eq!(source.backing_path(), zip_path.as_path());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn file_range_source_reports_size_and_reads_only_selected_window() {
        let root = temp_root("file-range-source");
        let bin_path = root.join("CharacterEditor.rdb.bin");
        std::fs::write(&bin_path, b"xxORIGINALyy").unwrap();
        let source = ReplacementSource::FileRange {
            path: bin_path,
            offset: 2,
            size: 8,
        };

        let mut bytes = Vec::new();
        source
            .open_reader()
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();

        assert_eq!(source.payload_size().unwrap(), 8);
        assert_eq!(bytes, b"ORIGINAL");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn memory_source_reports_size_and_reads_bytes() {
        let source = ReplacementSource::Memory {
            name: "CharacterEditor.rdb".to_string(),
            bytes: b"virtual-rdb".to_vec(),
        };
        let mut bytes = Vec::new();

        source
            .open_reader()
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();

        assert_eq!(source.payload_size().unwrap(), 11);
        assert_eq!(bytes, b"virtual-rdb");
        assert_eq!(source.backing_path(), Path::new("<memory>"));
    }

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "oppw4-replacement-source-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_zip(path: &Path, name: &str, bytes: &[u8]) {
        let file = File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer.start_file(name, options).unwrap();
        writer.write_all(bytes).unwrap();
        writer.finish().unwrap();
    }
}

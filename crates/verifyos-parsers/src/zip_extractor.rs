use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use zip::ZipArchive;

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Zip Error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Invalid IPA path")]
    InvalidPath,
}

pub struct ExtractedIpa {
    pub temp_dir: TempDir,
    pub payload_dir: PathBuf,
}

impl ExtractedIpa {
    pub fn get_app_bundle_path(&self) -> io::Result<Option<PathBuf>> {
        if !self.payload_dir.exists() {
            return Ok(None);
        }

        for entry in fs::read_dir(&self.payload_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }
}

pub fn extract_ipa<P: AsRef<Path>>(ipa_path: P) -> Result<ExtractedIpa, ExtractionError> {
    let file = fs::File::open(ipa_path)?;
    let mut archive = ZipArchive::new(file)?;
    let temp_dir = tempfile::tempdir()?;
    let extract_path = temp_dir.path();

    archive.extract(extract_path)?;

    let payload_dir = extract_path.join("Payload");

    Ok(ExtractedIpa {
        temp_dir,
        payload_dir,
    })
}

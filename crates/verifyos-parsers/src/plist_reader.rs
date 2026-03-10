use plist::Value;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PlistError {
    #[error("Plist Error: {0}")]
    ParseError(#[from] plist::Error),
    #[error("Not a dictionary")]
    NotADictionary,
}

pub struct InfoPlist {
    pub root: Value,
}

impl InfoPlist {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PlistError> {
        let value = Value::from_file(path)?;
        if value.as_dictionary().is_none() {
            return Err(PlistError::NotADictionary);
        }
        Ok(Self { root: value })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PlistError> {
        let value = Value::from_reader(std::io::Cursor::new(bytes))?;
        if value.as_dictionary().is_none() {
            return Err(PlistError::NotADictionary);
        }
        Ok(Self { root: value })
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.root
            .as_dictionary()
            .and_then(|dict| dict.get(key))
            .and_then(|v| v.as_string())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.root
            .as_dictionary()
            .and_then(|dict| dict.get(key))
            .and_then(|v| v.as_boolean())
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.root
            .as_dictionary()
            .map(|dict| dict.contains_key(key))
            .unwrap_or(false)
    }
}

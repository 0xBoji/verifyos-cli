use plist::Value;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PlistError {
    #[error("Plist Error: {0}")]
    ParseError(#[from] plist::Error),
    #[error("Not a dictionary")]
    NotADictionary,
}

#[derive(Debug, Clone)]
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

    pub fn from_dictionary(dict: plist::Dictionary) -> Self {
        Self {
            root: Value::Dictionary(dict),
        }
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

    pub fn get_dictionary(&self, key: &str) -> Option<&plist::Dictionary> {
        self.root
            .as_dictionary()
            .and_then(|dict| dict.get(key))
            .and_then(|v| v.as_dictionary())
    }

    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.root.as_dictionary().and_then(|dict| dict.get(key))
    }

    pub fn get_array_strings(&self, key: &str) -> Option<Vec<String>> {
        self.root
            .as_dictionary()
            .and_then(|dict| dict.get(key))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_string().map(|s| s.to_string()))
                    .collect()
            })
    }

    pub fn get_app_icons(&self) -> Vec<String> {
        let mut icons = Vec::new();

        // 1. Check CFBundleIcons
        if let Some(icons_dict) = self.get_dictionary("CFBundleIcons") {
            if let Some(primary_icon) = icons_dict
                .get("CFBundlePrimaryIcon")
                .and_then(|v| v.as_dictionary())
            {
                if let Some(files) = primary_icon
                    .get("CFBundleIconFiles")
                    .and_then(|v| v.as_array())
                {
                    for file in files {
                        if let Some(name) = file.as_string() {
                            icons.push(name.to_string());
                        }
                    }
                }
            }
        }

        // 2. Fallback to CFBundleIconFiles (older style)
        if icons.is_empty() {
            if let Some(files) = self.get_array_strings("CFBundleIconFiles") {
                icons.extend(files);
            }
        }

        // 3. Fallback to CFBundleIconFile (even older style)
        if icons.is_empty() {
            if let Some(file) = self.get_string("CFBundleIconFile") {
                icons.push(file.to_string());
            }
        }

        icons
    }
}

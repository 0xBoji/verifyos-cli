use crate::parsers::plist_reader::InfoPlist;
use std::path::Path;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ProvisioningError {
    #[error("Failed to read provisioning profile: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse provisioning profile plist: {0}")]
    Plist(#[from] crate::parsers::plist_reader::PlistError),
    #[error("Provisioning profile plist not found in CMS blob")]
    PlistNotFound,
    #[error("Provisioning profile missing Entitlements dictionary")]
    MissingEntitlements,
}

#[derive(Debug, Clone)]
pub struct ProvisioningProfile {
    pub entitlements: InfoPlist,
}

impl ProvisioningProfile {
    pub fn from_embedded_file<P: AsRef<Path>>(path: P) -> Result<Self, ProvisioningError> {
        let bytes = std::fs::read(path)?;
        let plist_bytes = extract_plist_bytes(&bytes)?;
        let profile_plist = InfoPlist::from_bytes(&plist_bytes)?;
        let entitlements_dict = profile_plist
            .get_dictionary("Entitlements")
            .ok_or(ProvisioningError::MissingEntitlements)?;

        Ok(Self {
            entitlements: InfoPlist::from_dictionary(entitlements_dict.clone()),
        })
    }
}

fn extract_plist_bytes(data: &[u8]) -> Result<Vec<u8>, ProvisioningError> {
    let start = find_subslice(data, b"<?xml").or_else(|| find_subslice(data, b"<plist"));
    let end = find_subslice(data, b"</plist>").map(|i| i + b"</plist>".len());

    match (start, end) {
        (Some(s), Some(e)) if e > s => Ok(data[s..e].to_vec()),
        _ => Err(ProvisioningError::PlistNotFound),
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

use std::path::Path;
use apple_codesign::MachFile;

#[derive(Debug, thiserror::Error)]
pub enum MachOError {
    #[error("Failed to parse Mach-O file: {0}")]
    ParseError(#[from] Box<apple_codesign::AppleCodesignError>),
    #[error("No code signature found")]
    NoSignature,
    #[error("Entitlements differ across Mach-O architecture slices")]
    MismatchedEntitlements,
}

pub struct MachOExecutable {
    pub entitlements: Option<String>,
}

impl MachOExecutable {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MachOError> {
        let file_data = std::fs::read(path).map_err(|e| Box::new(apple_codesign::AppleCodesignError::Io(e)))?;
        let mach_file = MachFile::parse(&file_data).map_err(Box::new)?;
        
        let mut first_entitlements: Option<Option<String>> = None;

        for macho in mach_file.iter_macho() {
            let mut current_ent = None;
            if let Some(sig) = macho.code_signature().map_err(Box::new)? {
                if let Some(ent) = sig.entitlements().map_err(Box::new)? {
                    current_ent = Some(ent.as_str().to_string());
                }
            }

            match &first_entitlements {
                None => {
                    first_entitlements = Some(current_ent);
                }
                Some(first) => {
                    if first != &current_ent {
                        return Err(MachOError::MismatchedEntitlements);
                    }
                }
            }
        }

        Ok(Self { 
            entitlements: first_entitlements.flatten() 
        })
    }
}

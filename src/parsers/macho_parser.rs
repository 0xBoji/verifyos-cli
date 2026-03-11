use apple_codesign::{CodeSignature, MachFile, SignatureEntity, SignatureReader};
use std::path::Path;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum MachOError {
    #[error("Failed to parse Mach-O file: {0}")]
    ParseError(#[from] Box<apple_codesign::AppleCodesignError>),
    #[error("Failed to read code signature data: {0}")]
    SignatureRead(#[source] Box<apple_codesign::AppleCodesignError>),
    #[error("No code signature found")]
    NoSignature,
    #[error("Entitlements differ across Mach-O architecture slices")]
    MismatchedEntitlements,
    #[error("Team identifiers differ across Mach-O architecture slices")]
    MismatchedTeamId,
}

pub struct MachOExecutable {
    pub entitlements: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MachOSignatureSummary {
    pub team_id: Option<String>,
    pub signed_slices: usize,
    pub total_slices: usize,
}

impl MachOExecutable {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MachOError> {
        let file_data =
            std::fs::read(path).map_err(|e| Box::new(apple_codesign::AppleCodesignError::Io(e)))?;
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
            entitlements: first_entitlements.flatten(),
        })
    }
}

pub fn read_macho_signature_summary<P: AsRef<Path>>(
    path: P,
) -> Result<MachOSignatureSummary, MachOError> {
    let reader = SignatureReader::from_path(path.as_ref())
        .map_err(|e| MachOError::SignatureRead(Box::new(e)))?;
    let entities = reader
        .entities()
        .map_err(|e| MachOError::SignatureRead(Box::new(e)))?;

    let mut team_ids = std::collections::BTreeSet::new();
    let mut total_slices = 0usize;
    let mut signed_slices = 0usize;

    for entity in entities {
        if let SignatureEntity::MachO(macho) = entity.entity {
            total_slices += 1;
            if let Some(signature) = macho.signature {
                signed_slices += 1;
                if let Some(team_id) = extract_team_id(&signature) {
                    team_ids.insert(team_id);
                }
            }
        }
    }

    if team_ids.len() > 1 {
        return Err(MachOError::MismatchedTeamId);
    }

    Ok(MachOSignatureSummary {
        team_id: team_ids.into_iter().next(),
        signed_slices,
        total_slices,
    })
}

fn extract_team_id(signature: &CodeSignature) -> Option<String> {
    if let Some(code_directory) = &signature.code_directory {
        if let Some(team_id) = &code_directory.team_name {
            return Some(team_id.clone());
        }
    }

    if let Some(cms) = &signature.cms {
        for cert in &cms.certificates {
            if let Some(team_id) = &cert.apple_team_id {
                return Some(team_id.clone());
            }
        }
    }

    None
}

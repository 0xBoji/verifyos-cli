use miette::Diagnostic;
use serde::{Deserialize, Serialize};

pub const RULESET_VERSION: &str = "0.1.0";

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum RuleError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Entitlements(#[from] crate::rules::entitlements::EntitlementsError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Provisioning(#[from] crate::parsers::provisioning_profile::ProvisioningError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    MachO(#[from] crate::parsers::macho_parser::MachOError),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleStatus {
    Pass,
    Fail,
    Error,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleReport {
    pub status: RuleStatus,
    pub message: Option<String>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RuleCategory {
    Privacy,
    Signing,
    Bundling,
    Entitlements,
    Ats,
    ThirdParty,
    Permissions,
    Metadata,
    Other,
}

// Stub for now. Will hold the path to the app and the parsed Info.plist
pub struct ArtifactContext<'a> {
    pub app_bundle_path: &'a std::path::Path,
    pub info_plist: Option<&'a crate::parsers::plist_reader::InfoPlist>,
}

pub trait AppStoreRule {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> RuleCategory;
    fn severity(&self) -> Severity;
    fn recommendation(&self) -> &'static str;
    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError>;
}

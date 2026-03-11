use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum RuleError {
    #[error("Missing Privacy Manifest")]
    #[diagnostic(
        code(verifyos::privacy::missing_manifest),
        help("Apple requires a PrivacyInfo.xcprivacy file for apps. Please include one in your app bundle.")
    )]
    MissingPrivacyManifest,

    #[error("Missing Camera Usage Description")]
    #[diagnostic(
        code(verifyos::permissions::missing_camera_desc),
        help("The Info.plist is missing the NSCameraUsageDescription key.")
    )]
    MissingCameraUsageDescription,

    #[error(transparent)]
    #[diagnostic(transparent)]
    Entitlements(#[from] crate::rules::entitlements::EntitlementsError),
}

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
pub struct RuleResult {
    pub success: bool,
}

// Stub for now. Will hold the path to the app and the parsed Info.plist
pub struct ArtifactContext<'a> {
    pub app_bundle_path: &'a std::path::Path,
    pub info_plist: Option<&'a crate::parsers::plist_reader::InfoPlist>,
}

pub trait AppStoreRule {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleResult, RuleError>;
}

use crate::parsers::bundle_scanner::{find_nested_bundles, BundleScanError, BundleTarget};
use crate::parsers::macho_parser::{
    read_macho_signature_summary, MachOError, MachOSignatureSummary,
};
use crate::parsers::macho_scanner::{
    scan_capabilities_from_app_bundle, scan_private_api_from_app_bundle, scan_sdks_from_app_bundle,
    scan_usage_from_app_bundle, CapabilityScan, PrivateApiScan, SdkScan, UsageScan, UsageScanError,
};
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    nested_bundles_cache: RefCell<Option<Vec<BundleTarget>>>,
    usage_scan_cache: RefCell<Option<UsageScan>>,
    private_api_scan_cache: RefCell<Option<PrivateApiScan>>,
    sdk_scan_cache: RefCell<Option<SdkScan>>,
    capability_scan_cache: RefCell<Option<CapabilityScan>>,
    signature_summary_cache: RefCell<HashMap<PathBuf, MachOSignatureSummary>>,
}

impl<'a> ArtifactContext<'a> {
    pub fn new(
        app_bundle_path: &'a Path,
        info_plist: Option<&'a crate::parsers::plist_reader::InfoPlist>,
    ) -> Self {
        Self {
            app_bundle_path,
            info_plist,
            nested_bundles_cache: RefCell::new(None),
            usage_scan_cache: RefCell::new(None),
            private_api_scan_cache: RefCell::new(None),
            sdk_scan_cache: RefCell::new(None),
            capability_scan_cache: RefCell::new(None),
            signature_summary_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn nested_bundles(&self) -> Result<Vec<BundleTarget>, BundleScanError> {
        if let Some(bundles) = self.nested_bundles_cache.borrow().as_ref() {
            return Ok(bundles.clone());
        }

        let bundles = find_nested_bundles(self.app_bundle_path)?;
        *self.nested_bundles_cache.borrow_mut() = Some(bundles.clone());
        Ok(bundles)
    }

    pub fn usage_scan(&self) -> Result<UsageScan, UsageScanError> {
        if let Some(scan) = self.usage_scan_cache.borrow().as_ref() {
            return Ok(scan.clone());
        }

        let scan = scan_usage_from_app_bundle(self.app_bundle_path)?;
        *self.usage_scan_cache.borrow_mut() = Some(scan.clone());
        Ok(scan)
    }

    pub fn private_api_scan(&self) -> Result<PrivateApiScan, UsageScanError> {
        if let Some(scan) = self.private_api_scan_cache.borrow().as_ref() {
            return Ok(scan.clone());
        }

        let scan = scan_private_api_from_app_bundle(self.app_bundle_path)?;
        *self.private_api_scan_cache.borrow_mut() = Some(scan.clone());
        Ok(scan)
    }

    pub fn sdk_scan(&self) -> Result<SdkScan, UsageScanError> {
        if let Some(scan) = self.sdk_scan_cache.borrow().as_ref() {
            return Ok(scan.clone());
        }

        let scan = scan_sdks_from_app_bundle(self.app_bundle_path)?;
        *self.sdk_scan_cache.borrow_mut() = Some(scan.clone());
        Ok(scan)
    }

    pub fn capability_scan(&self) -> Result<CapabilityScan, UsageScanError> {
        if let Some(scan) = self.capability_scan_cache.borrow().as_ref() {
            return Ok(scan.clone());
        }

        let scan = scan_capabilities_from_app_bundle(self.app_bundle_path)?;
        *self.capability_scan_cache.borrow_mut() = Some(scan.clone());
        Ok(scan)
    }

    pub fn signature_summary(
        &self,
        executable_path: impl AsRef<Path>,
    ) -> Result<MachOSignatureSummary, MachOError> {
        let executable_path = executable_path.as_ref().to_path_buf();
        if let Some(summary) = self.signature_summary_cache.borrow().get(&executable_path) {
            return Ok(summary.clone());
        }

        let summary = read_macho_signature_summary(&executable_path)?;
        self.signature_summary_cache
            .borrow_mut()
            .insert(executable_path, summary.clone());
        Ok(summary)
    }
}

pub trait AppStoreRule {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> RuleCategory;
    fn severity(&self) -> Severity;
    fn recommendation(&self) -> &'static str;
    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError>;
}

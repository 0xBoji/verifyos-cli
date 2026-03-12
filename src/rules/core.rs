use crate::parsers::bundle_scanner::{find_nested_bundles, BundleScanError, BundleTarget};
use crate::parsers::macho_parser::{
    read_macho_signature_summary, MachOError, MachOExecutable, MachOSignatureSummary,
};
use crate::parsers::macho_scanner::{
    scan_capabilities_from_app_bundle, scan_private_api_from_app_bundle, scan_sdks_from_app_bundle,
    scan_usage_from_app_bundle, CapabilityScan, PrivateApiScan, SdkScan, UsageScan, UsageScanError,
};
use crate::parsers::plist_reader::{InfoPlist, PlistError};
use crate::parsers::provisioning_profile::{ProvisioningError, ProvisioningProfile};
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
    bundle_plist_cache: RefCell<HashMap<PathBuf, Option<InfoPlist>>>,
    entitlements_cache: RefCell<HashMap<PathBuf, Option<InfoPlist>>>,
    provisioning_profile_cache: RefCell<HashMap<PathBuf, Option<ProvisioningProfile>>>,
    bundle_file_cache: RefCell<Option<Vec<PathBuf>>>,
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
            bundle_plist_cache: RefCell::new(HashMap::new()),
            entitlements_cache: RefCell::new(HashMap::new()),
            provisioning_profile_cache: RefCell::new(HashMap::new()),
            bundle_file_cache: RefCell::new(None),
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

    pub fn executable_path_for_bundle(&self, bundle_path: &Path) -> Option<PathBuf> {
        if let Ok(Some(plist)) = self.bundle_info_plist(bundle_path) {
            if let Some(executable) = plist.get_string("CFBundleExecutable") {
                let candidate = bundle_path.join(executable);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        resolve_bundle_executable_path(bundle_path)
    }

    pub fn bundle_info_plist(&self, bundle_path: &Path) -> Result<Option<InfoPlist>, PlistError> {
        if let Some(plist) = self.bundle_plist_cache.borrow().get(bundle_path) {
            return Ok(plist.clone());
        }

        let plist_path = bundle_path.join("Info.plist");
        let plist = if plist_path.exists() {
            Some(InfoPlist::from_file(&plist_path)?)
        } else {
            None
        };

        self.bundle_plist_cache
            .borrow_mut()
            .insert(bundle_path.to_path_buf(), plist.clone());
        Ok(plist)
    }

    pub fn entitlements_for_bundle(
        &self,
        bundle_path: &Path,
    ) -> Result<Option<InfoPlist>, RuleError> {
        let executable_path = match self.executable_path_for_bundle(bundle_path) {
            Some(path) => path,
            None => return Ok(None),
        };

        if let Some(entitlements) = self.entitlements_cache.borrow().get(&executable_path) {
            return Ok(entitlements.clone());
        }

        let macho = MachOExecutable::from_file(&executable_path)
            .map_err(crate::rules::entitlements::EntitlementsError::MachO)
            .map_err(RuleError::Entitlements)?;
        let entitlements = match macho.entitlements {
            Some(entitlements_xml) => {
                let plist = InfoPlist::from_bytes(entitlements_xml.as_bytes())
                    .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;
                Some(plist)
            }
            None => None,
        };

        self.entitlements_cache
            .borrow_mut()
            .insert(executable_path, entitlements.clone());
        Ok(entitlements)
    }

    pub fn provisioning_profile_for_bundle(
        &self,
        bundle_path: &Path,
    ) -> Result<Option<ProvisioningProfile>, ProvisioningError> {
        let provisioning_path = bundle_path.join("embedded.mobileprovision");
        if let Some(profile) = self
            .provisioning_profile_cache
            .borrow()
            .get(&provisioning_path)
        {
            return Ok(profile.clone());
        }

        let profile = if provisioning_path.exists() {
            Some(ProvisioningProfile::from_embedded_file(&provisioning_path)?)
        } else {
            None
        };

        self.provisioning_profile_cache
            .borrow_mut()
            .insert(provisioning_path, profile.clone());
        Ok(profile)
    }

    pub fn bundle_file_paths(&self) -> Vec<PathBuf> {
        if let Some(paths) = self.bundle_file_cache.borrow().as_ref() {
            return paths.clone();
        }

        let mut files = Vec::new();
        collect_bundle_files(self.app_bundle_path, &mut files);
        *self.bundle_file_cache.borrow_mut() = Some(files.clone());
        files
    }

    pub fn bundle_relative_file(&self, relative_path: &str) -> Option<PathBuf> {
        self.bundle_file_paths().into_iter().find(|path| {
            path.strip_prefix(self.app_bundle_path)
                .ok()
                .map(|rel| rel == Path::new(relative_path))
                .unwrap_or(false)
        })
    }
}

fn resolve_bundle_executable_path(bundle_path: &Path) -> Option<PathBuf> {
    let bundle_name = bundle_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches(".app")
        .trim_end_matches(".appex")
        .trim_end_matches(".framework");

    if bundle_name.is_empty() {
        return None;
    }

    let fallback = bundle_path.join(bundle_name);
    if fallback.exists() {
        Some(fallback)
    } else {
        None
    }
}

fn collect_bundle_files(root: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_bundle_files(&path, files);
        } else {
            files.push(path);
        }
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

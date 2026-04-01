use crate::parsers::plist_reader::InfoPlist;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum UsageScanError {
    #[error("Failed to read executable: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unable to resolve app executable")]
    MissingExecutable,
}

#[derive(Debug, Default, Clone)]
pub struct UsageScan {
    pub required_keys: HashSet<&'static str>,
    pub privacy_categories: HashSet<&'static str>,
    pub requires_location_key: bool,
    pub evidence: HashSet<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct CapabilityScan {
    pub detected: HashSet<&'static str>,
    pub evidence: HashSet<&'static str>,
}

pub fn scan_usage_from_app_bundle(app_bundle_path: &Path) -> Result<UsageScan, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    scan_usage_from_executable(&executable)
}

#[derive(Debug, Default, Clone)]
pub struct PrivateApiScan {
    pub hits: Vec<&'static str>,
}

#[derive(Debug, Default, Clone)]
pub struct SdkScan {
    pub hits: Vec<&'static str>,
}

pub fn scan_private_api_from_app_bundle(
    app_bundle_path: &Path,
) -> Result<PrivateApiScan, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    scan_private_api_from_executable(&executable)
}

pub fn scan_sdks_from_app_bundle(app_bundle_path: &Path) -> Result<SdkScan, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    scan_sdks_from_executable(&executable)
}

pub fn scan_capabilities_from_app_bundle(
    app_bundle_path: &Path,
) -> Result<CapabilityScan, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    scan_capabilities_from_executable(&executable)
}

pub fn scan_instrumentation_from_app_bundle(
    app_bundle_path: &Path,
) -> Result<Vec<&'static str>, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    let bytes = std::fs::read(&executable)?;
    let mut hits = Vec::new();
    for signature in INSTRUMENTATION_SIGNATURES {
        if contains_subslice(&bytes, signature.as_bytes()) {
            hits.push(*signature);
        }
    }
    Ok(hits)
}

fn resolve_executable_path(app_bundle_path: &Path) -> Option<PathBuf> {
    let info_plist_path = app_bundle_path.join("Info.plist");
    if info_plist_path.exists() {
        if let Ok(info_plist) = InfoPlist::from_file(&info_plist_path) {
            if let Some(executable) = info_plist.get_string("CFBundleExecutable") {
                let candidate = app_bundle_path.join(executable);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    let app_name = app_bundle_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches(".app");

    if app_name.is_empty() {
        return None;
    }

    let fallback = app_bundle_path.join(app_name);
    fallback.exists().then_some(fallback)
}

fn scan_usage_from_executable(path: &Path) -> Result<UsageScan, UsageScanError> {
    let bytes = std::fs::read(path)?;
    let mut scan = UsageScan::default();

    for (signature, requirement) in SIGNATURES {
        if contains_subslice(&bytes, signature.as_bytes()) {
            scan.evidence.insert(*signature);
            match requirement {
                Requirement::Key(key) => {
                    scan.required_keys.insert(*key);
                }
                Requirement::PrivacyCategory(cat) => {
                    scan.privacy_categories.insert(*cat);
                }
                Requirement::AnyLocation => {
                    scan.requires_location_key = true;
                }
            }
        }
    }

    Ok(scan)
}

fn scan_private_api_from_executable(path: &Path) -> Result<PrivateApiScan, UsageScanError> {
    let bytes = std::fs::read(path)?;
    let mut scan = PrivateApiScan::default();

    for signature in PRIVATE_API_SIGNATURES {
        if contains_subslice(&bytes, signature.as_bytes()) {
            scan.hits.push(*signature);
        }
    }

    scan.hits.sort_unstable();
    scan.hits.dedup();

    Ok(scan)
}

fn scan_sdks_from_executable(path: &Path) -> Result<SdkScan, UsageScanError> {
    let bytes = std::fs::read(path)?;
    let mut scan = SdkScan::default();

    for signature in SDK_SIGNATURES {
        if contains_subslice(&bytes, signature.as_bytes()) {
            scan.hits.push(*signature);
        }
    }

    scan.hits.sort_unstable();
    scan.hits.dedup();

    Ok(scan)
}

fn scan_capabilities_from_executable(path: &Path) -> Result<CapabilityScan, UsageScanError> {
    let bytes = std::fs::read(path)?;
    let mut scan = CapabilityScan::default();

    for (signature, capability) in CAPABILITY_SIGNATURES {
        if contains_subslice(&bytes, signature.as_bytes()) {
            scan.evidence.insert(*signature);
            scan.detected.insert(*capability);
        }
    }

    Ok(scan)
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[derive(Clone, Copy)]
enum Requirement {
    Key(&'static str),
    PrivacyCategory(&'static str),
    AnyLocation,
}

const SIGNATURES: &[(&str, Requirement)] = &[
    (
        "AVCaptureDevice",
        Requirement::Key("NSCameraUsageDescription"),
    ),
    (
        "AVAudioSession",
        Requirement::Key("NSMicrophoneUsageDescription"),
    ),
    (
        "AVAudioRecorder",
        Requirement::Key("NSMicrophoneUsageDescription"),
    ),
    (
        "PHPhotoLibrary",
        Requirement::Key("NSPhotoLibraryUsageDescription"),
    ),
    (
        "PHPhotoLibraryAddOnly",
        Requirement::Key("NSPhotoLibraryAddUsageDescription"),
    ),
    ("CLLocationManager", Requirement::AnyLocation),
    (
        "CBCentralManager",
        Requirement::Key("NSBluetoothAlwaysUsageDescription"),
    ),
    (
        "CBPeripheralManager",
        Requirement::Key("NSBluetoothAlwaysUsageDescription"),
    ),
    (
        "CBPeripheral",
        Requirement::Key("NSBluetoothPeripheralUsageDescription"),
    ),
    ("LAContext", Requirement::Key("NSFaceIDUsageDescription")),
    (
        "EKEventStore",
        Requirement::Key("NSCalendarsUsageDescription"),
    ),
    (
        "EKReminder",
        Requirement::Key("NSRemindersUsageDescription"),
    ),
    (
        "CNContactStore",
        Requirement::Key("NSContactsUsageDescription"),
    ),
    (
        "SFSpeechRecognizer",
        Requirement::Key("NSSpeechRecognitionUsageDescription"),
    ),
    (
        "CMMotionManager",
        Requirement::Key("NSMotionUsageDescription"),
    ),
    ("CMPedometer", Requirement::Key("NSMotionUsageDescription")),
    (
        "MPMediaLibrary",
        Requirement::Key("NSAppleMusicUsageDescription"),
    ),
    (
        "HKHealthStore",
        Requirement::Key("NSHealthShareUsageDescription"),
    ),
    // Required Reason APIs (2024 Mandate)
    (
        "systemBootTime",
        Requirement::PrivacyCategory("NSPrivacyAccessedAPICategorySystemBootTime"),
    ),
    (
        "diskSpace",
        Requirement::PrivacyCategory("NSPrivacyAccessedAPICategoryDiskSpace"),
    ),
    (
        "activeInputModes",
        Requirement::PrivacyCategory("NSPrivacyAccessedAPICategoryActiveInputModes"),
    ),
    (
        "userDefaults",
        Requirement::PrivacyCategory("NSPrivacyAccessedAPICategoryUserDefaults"),
    ),
    (
        "fileModificationDate",
        Requirement::PrivacyCategory("NSPrivacyAccessedAPICategoryFileModificationDate"),
    ),
];

const PRIVATE_API_SIGNATURES: &[&str] = &[
    "LSApplicationWorkspace",
    "LSApplicationProxy",
    "LSAppWorkspace",
    "SBApplication",
    "SpringBoard",
    "MobileGestalt",
    "UICallApplication",
    "UIGetScreenImage",
    "_MGCopyAnswer",
];

const SDK_SIGNATURES: &[&str] = &[
    "FirebaseApp",
    "FIRApp",
    "GADMobileAds",
    "FBSDKCoreKit",
    "FBSDKLoginKit",
    "Amplitude",
    "Mixpanel",
    "Segment",
    "SentrySDK",
    "AppsFlyerLib",
    "Adjust",
];

const CAPABILITY_SIGNATURES: &[(&str, &str)] = &[
    ("AVCaptureDevice", "camera"),
    ("CLLocationManager", "location"),
];

const INSTRUMENTATION_SIGNATURES: &[&str] = &[
    "__llvm_profile_runtime",
    "__llvm_prf_data",
    "__llvm_prf_names",
    "__llvm_prf_vnds",
];

#[cfg(test)]
mod tests {
    use super::{resolve_executable_path, scan_usage_from_app_bundle};
    use plist::{Dictionary, Value};
    use tempfile::tempdir;

    #[test]
    fn resolves_executable_from_cf_bundle_executable_before_bundle_name() {
        let dir = tempdir().expect("temp dir");
        let app_path = dir.path().join("CustomName.app");
        std::fs::create_dir_all(&app_path).expect("create app dir");

        let mut dict = Dictionary::new();
        dict.insert(
            "CFBundleExecutable".to_string(),
            Value::String("RunnerBinary".to_string()),
        );
        Value::Dictionary(dict)
            .to_file_xml(app_path.join("Info.plist"))
            .expect("write plist");
        std::fs::write(app_path.join("RunnerBinary"), b"plain binary").expect("write executable");

        let resolved = resolve_executable_path(&app_path).expect("resolved executable");
        assert_eq!(resolved, app_path.join("RunnerBinary"));
    }

    #[test]
    fn usage_scan_reads_custom_executable_name_from_info_plist() {
        let dir = tempdir().expect("temp dir");
        let app_path = dir.path().join("CustomName.app");
        std::fs::create_dir_all(&app_path).expect("create app dir");

        let mut dict = Dictionary::new();
        dict.insert(
            "CFBundleExecutable".to_string(),
            Value::String("RunnerBinary".to_string()),
        );
        Value::Dictionary(dict)
            .to_file_xml(app_path.join("Info.plist"))
            .expect("write plist");
        std::fs::write(app_path.join("RunnerBinary"), b"AVCaptureDevice")
            .expect("write executable");

        let scan = scan_usage_from_app_bundle(&app_path).expect("usage scan");
        assert!(scan.required_keys.contains("NSCameraUsageDescription"));
        assert!(scan.evidence.contains("AVCaptureDevice"));
    }
}

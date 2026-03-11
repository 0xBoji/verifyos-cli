use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum UsageScanError {
    #[error("Failed to read executable: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unable to resolve app executable")]
    MissingExecutable,
}

#[derive(Debug, Default)]
pub struct UsageScan {
    pub required_keys: HashSet<&'static str>,
    pub requires_location_key: bool,
    pub evidence: HashSet<&'static str>,
}

pub fn scan_usage_from_app_bundle(app_bundle_path: &Path) -> Result<UsageScan, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    scan_usage_from_executable(&executable)
}

#[derive(Debug, Default)]
pub struct PrivateApiScan {
    pub hits: Vec<&'static str>,
}

pub fn scan_private_api_from_app_bundle(
    app_bundle_path: &Path,
) -> Result<PrivateApiScan, UsageScanError> {
    let executable =
        resolve_executable_path(app_bundle_path).ok_or(UsageScanError::MissingExecutable)?;
    scan_private_api_from_executable(&executable)
}

fn resolve_executable_path(app_bundle_path: &Path) -> Option<PathBuf> {
    let app_name = app_bundle_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches(".app");

    if app_name.is_empty() {
        return None;
    }

    let executable_path = app_bundle_path.join(app_name);
    if executable_path.exists() {
        Some(executable_path)
    } else {
        None
    }
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

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[derive(Clone, Copy)]
enum Requirement {
    Key(&'static str),
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

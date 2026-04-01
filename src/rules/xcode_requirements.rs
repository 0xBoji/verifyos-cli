use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_STORE_CONNECT_REQUIREMENT_DATE: &str = "April 28, 2026";
const APP_STORE_CONNECT_REQUIREMENT_UNIX: u64 = 1_777_334_400;
const MIN_XCODE_BUILD_NUMBER: u32 = 2600;
const MIN_IOS_SDK_MAJOR: u32 = 26;

pub struct XcodeVersionRule;

impl AppStoreRule for XcodeVersionRule {
    fn id(&self) -> &'static str {
        "RULE_XCODE_26_MANDATE"
    }

    fn name(&self) -> &'static str {
        "Xcode 26 / iOS 26 SDK Mandate"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Starting April 28, 2026, iOS and iPadOS apps uploaded to App Store Connect must be built with Xcode 26 and the iOS 26 SDK or later."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let mut failures = Vec::new();

        match plist.get_string("DTXcode") {
            Some(version) => match version.parse::<u32>() {
                Ok(build_number) if build_number >= MIN_XCODE_BUILD_NUMBER => {}
                Ok(_) => failures.push(format!(
                    "DTXcode={} is below the minimum build number {}",
                    version, MIN_XCODE_BUILD_NUMBER
                )),
                Err(_) => failures.push(format!("DTXcode={} is not a valid build number", version)),
            },
            None => failures.push("DTXcode key missing".to_string()),
        }

        match plist.get_string("DTPlatformVersion") {
            Some(version) => match parse_major_version(version) {
                Some(major) if major >= MIN_IOS_SDK_MAJOR => {}
                Some(_) => failures.push(format!(
                    "DTPlatformVersion={} is below {}.0",
                    version, MIN_IOS_SDK_MAJOR
                )),
                None => failures.push(format!(
                    "DTPlatformVersion={} is not a valid platform version",
                    version
                )),
            },
            None => failures.push("DTPlatformVersion key missing".to_string()),
        }

        match plist.get_string("DTSDKName") {
            Some(name) => match parse_sdk_major_version(name) {
                Some(major) if major >= MIN_IOS_SDK_MAJOR => {}
                Some(_) => failures.push(format!(
                    "DTSDKName={} is below iphoneos{}",
                    name, MIN_IOS_SDK_MAJOR
                )),
                None => failures.push(format!(
                    "DTSDKName={} does not look like an iPhone OS SDK identifier",
                    name
                )),
            },
            None => failures.push("DTSDKName key missing".to_string()),
        }

        if failures.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some(success_message().to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some(failure_message().to_string()),
            evidence: Some(failures.join("; ")),
        })
    }
}

fn parse_major_version(value: &str) -> Option<u32> {
    value.trim().split('.').next()?.parse::<u32>().ok()
}

fn parse_sdk_major_version(value: &str) -> Option<u32> {
    let suffix = value.trim().strip_prefix("iphoneos")?;
    parse_major_version(suffix)
}

fn requirement_is_live() -> bool {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() >= APP_STORE_CONNECT_REQUIREMENT_UNIX)
        .unwrap_or(true)
}

fn success_message() -> &'static str {
    if requirement_is_live() {
        "App meets the current App Store Connect Xcode 26 / iOS 26 SDK requirement"
    } else {
        "App already meets the upcoming App Store Connect Xcode 26 / iOS 26 SDK requirement"
    }
}

fn failure_message() -> String {
    if requirement_is_live() {
        "App does not meet the current App Store Connect Xcode 26 / iOS 26 SDK requirement"
            .to_string()
    } else {
        format!(
            "App does not meet the upcoming App Store Connect requirement that takes effect on {APP_STORE_CONNECT_REQUIREMENT_DATE}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_major_version, parse_sdk_major_version};

    #[test]
    fn parses_major_platform_version() {
        assert_eq!(parse_major_version("26.1"), Some(26));
        assert_eq!(parse_major_version("26"), Some(26));
        assert_eq!(parse_major_version(""), None);
        assert_eq!(parse_major_version("abc"), None);
    }

    #[test]
    fn parses_sdk_major_version() {
        assert_eq!(parse_sdk_major_version("iphoneos26.0"), Some(26));
        assert_eq!(parse_sdk_major_version("iphoneos26"), Some(26));
        assert_eq!(parse_sdk_major_version("iphonesimulator26.0"), None);
        assert_eq!(parse_sdk_major_version("iphoneos"), None);
    }
}

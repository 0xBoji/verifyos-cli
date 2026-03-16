use serde::{Deserialize, Serialize};
use verifyos_cli::report::ReportData;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanProfileInput {
    Basic,
    Full,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub profile: Option<ScanProfileInput>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub baseline: Option<ReportData>,
}

#[derive(Debug, Deserialize)]
pub struct AuthStartRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStartResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthVerifyRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct AuthVerifyResponse {
    pub token: String,
    pub email: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct BaselineInfo {
    pub suppressed: usize,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub report: ReportData,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineInfo>,
}

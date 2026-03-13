use crate::core::engine::Engine;
use crate::rules::ats::{AtsAuditRule, AtsExceptionsGranularityRule};
use crate::rules::bundle_leakage::BundleResourceLeakageRule;
use crate::rules::bundle_metadata::BundleMetadataConsistencyRule;
use crate::rules::core::AppStoreRule;
use crate::rules::core::{RuleCategory, Severity};
use crate::rules::entitlements::{EntitlementsMismatchRule, EntitlementsProvisioningMismatchRule};
use crate::rules::export_compliance::ExportComplianceRule;
use crate::rules::extensions::ExtensionEntitlementsCompatibilityRule;
use crate::rules::info_plist::{
    InfoPlistCapabilitiesRule, InfoPlistRequiredKeysRule, InfoPlistVersionConsistencyRule,
    LSApplicationQueriesSchemesAuditRule, UIRequiredDeviceCapabilitiesAuditRule,
    UsageDescriptionsRule, UsageDescriptionsValueRule,
};
use crate::rules::permissions::CameraUsageDescriptionRule;
use crate::rules::privacy::MissingPrivacyManifestRule;
use crate::rules::privacy_manifest::PrivacyManifestCompletenessRule;
use crate::rules::privacy_sdk::PrivacyManifestSdkCrossCheckRule;
use crate::rules::private_api::PrivateApiRule;
use crate::rules::signing::EmbeddedCodeSignatureTeamRule;
use clap::ValueEnum;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanProfile {
    Basic,
    Full,
}

#[derive(Debug, Clone, Default)]
pub struct RuleSelection {
    pub include: HashSet<String>,
    pub exclude: HashSet<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleInventoryItem {
    pub rule_id: String,
    pub name: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub default_profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleDetailItem {
    pub rule_id: String,
    pub name: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub recommendation: String,
    pub default_profiles: Vec<String>,
}

impl RuleSelection {
    pub fn allows(&self, rule_id: &str) -> bool {
        let normalized = normalize_rule_id(rule_id);
        let included = self.include.is_empty() || self.include.contains(&normalized);
        let excluded = self.exclude.contains(&normalized);
        included && !excluded
    }
}

pub fn register_rules(engine: &mut Engine, profile: ScanProfile, selection: &RuleSelection) {
    for rule in profile_rules(profile) {
        if selection.allows(rule.id()) {
            engine.register_rule(rule);
        }
    }
}

pub fn available_rule_ids(profile: ScanProfile) -> Vec<String> {
    let mut ids: Vec<String> = profile_rules(profile)
        .into_iter()
        .map(|rule| normalize_rule_id(rule.id()))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

pub fn normalize_rule_id(rule_id: &str) -> String {
    rule_id.trim().to_ascii_uppercase()
}

pub fn rule_inventory() -> Vec<RuleInventoryItem> {
    let mut items: BTreeMap<String, RuleInventoryItem> = BTreeMap::new();

    for (profile_name, profile) in [("basic", ScanProfile::Basic), ("full", ScanProfile::Full)] {
        for rule in profile_rules(profile) {
            let rule_id = normalize_rule_id(rule.id());
            let entry = items
                .entry(rule_id.clone())
                .or_insert_with(|| RuleInventoryItem {
                    rule_id,
                    name: rule.name().to_string(),
                    severity: rule.severity(),
                    category: rule.category(),
                    default_profiles: Vec::new(),
                });

            if !entry
                .default_profiles
                .iter()
                .any(|name| name == profile_name)
            {
                entry.default_profiles.push(profile_name.to_string());
            }
        }
    }

    items.into_values().collect()
}

pub fn rule_detail(rule_id: &str) -> Option<RuleDetailItem> {
    let normalized = normalize_rule_id(rule_id);
    let mut detail: Option<RuleDetailItem> = None;

    for (profile_name, profile) in [("basic", ScanProfile::Basic), ("full", ScanProfile::Full)] {
        for rule in profile_rules(profile) {
            if normalize_rule_id(rule.id()) != normalized {
                continue;
            }

            let entry = detail.get_or_insert_with(|| RuleDetailItem {
                rule_id: normalize_rule_id(rule.id()),
                name: rule.name().to_string(),
                severity: rule.severity(),
                category: rule.category(),
                recommendation: rule.recommendation().to_string(),
                default_profiles: Vec::new(),
            });

            if !entry
                .default_profiles
                .iter()
                .any(|name| name == profile_name)
            {
                entry.default_profiles.push(profile_name.to_string());
            }
        }
    }

    detail
}

fn profile_rules(profile: ScanProfile) -> Vec<Box<dyn AppStoreRule>> {
    match profile {
        ScanProfile::Basic => basic_rules(),
        ScanProfile::Full => full_rules(),
    }
}

fn basic_rules() -> Vec<Box<dyn AppStoreRule>> {
    vec![
        Box::new(MissingPrivacyManifestRule),
        Box::new(UsageDescriptionsRule),
        Box::new(UsageDescriptionsValueRule),
        Box::new(CameraUsageDescriptionRule),
        Box::new(AtsAuditRule),
        Box::new(AtsExceptionsGranularityRule),
        Box::new(EntitlementsMismatchRule),
        Box::new(EntitlementsProvisioningMismatchRule),
        Box::new(EmbeddedCodeSignatureTeamRule),
    ]
}

fn full_rules() -> Vec<Box<dyn AppStoreRule>> {
    vec![
        Box::new(MissingPrivacyManifestRule),
        Box::new(PrivacyManifestCompletenessRule),
        Box::new(PrivacyManifestSdkCrossCheckRule),
        Box::new(CameraUsageDescriptionRule),
        Box::new(UsageDescriptionsRule),
        Box::new(UsageDescriptionsValueRule),
        Box::new(InfoPlistRequiredKeysRule),
        Box::new(InfoPlistCapabilitiesRule),
        Box::new(LSApplicationQueriesSchemesAuditRule),
        Box::new(UIRequiredDeviceCapabilitiesAuditRule),
        Box::new(InfoPlistVersionConsistencyRule),
        Box::new(ExportComplianceRule),
        Box::new(AtsAuditRule),
        Box::new(AtsExceptionsGranularityRule),
        Box::new(EntitlementsMismatchRule),
        Box::new(EntitlementsProvisioningMismatchRule),
        Box::new(BundleMetadataConsistencyRule),
        Box::new(BundleResourceLeakageRule),
        Box::new(ExtensionEntitlementsCompatibilityRule),
        Box::new(PrivateApiRule),
        Box::new(EmbeddedCodeSignatureTeamRule),
    ]
}

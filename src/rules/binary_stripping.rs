use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use goblin::mach::Mach;
use std::path::Path;

pub struct BinaryStrippingRule;

impl AppStoreRule for BinaryStrippingRule {
    fn id(&self) -> &'static str {
        "RULE_BINARY_STRIPPING"
    }

    fn name(&self) -> &'static str {
        "Binary Stripping & Instrumentation Check"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Bundling
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Ensure your binary is stripped of debug symbols and LLVM profiling instrumentation in production builds."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(executable_path) = artifact.executable_path_for_bundle(artifact.app_bundle_path)
        else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Main executable not found".to_string()),
                evidence: None,
            });
        };

        let mut issues = Vec::new();

        // 1. Check for LLVM instrumentation (from macho_scanner)
        if let Ok(hits) = artifact.instrumentation_scan() {
            if !hits.is_empty() {
                issues.push(format!(
                    "Leftover LLVM instrumentation detected: {}",
                    hits.join(", ")
                ));
            }
        }

        // 2. Check for symbol table using goblin
        match check_is_stripped(&executable_path) {
            Ok(false) => {
                issues.push("Binary contains a symbol table (not fully stripped)".to_string());
            }
            Ok(true) => {}
            Err(e) => {
                return Ok(RuleReport {
                    status: RuleStatus::Error,
                    message: Some(format!("Failed to analyze binary symbols: {e}")),
                    evidence: None,
                });
            }
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("Binary is stripped and free of instrumentation".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Binary hygiene issues detected".to_string()),
            evidence: Some(issues.join(" | ")),
        })
    }
}

fn check_is_stripped(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let buffer = std::fs::read(path)?;
    match Mach::parse(&buffer)? {
        Mach::Binary(macho) => {
            // Check for symbol table in load commands
            for lc in &macho.load_commands {
                if let goblin::mach::load_command::CommandVariant::Symtab(symtab) = lc.command {
                    if symtab.nsyms > 0 {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
        Mach::Fat(fat) => {
            // Check all architectures
            for arch in fat.iter_arches() {
                let arch = arch?;
                let macho = goblin::mach::MachO::parse(
                    &buffer[arch.offset as usize..(arch.offset + arch.size) as usize],
                    0,
                )?;
                for lc in &macho.load_commands {
                    if let goblin::mach::load_command::CommandVariant::Symtab(symtab) = lc.command {
                        if symtab.nsyms > 0 {
                            return Ok(false);
                        }
                    }
                }
            }
            Ok(true)
        }
    }
}

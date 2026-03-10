use clap::Parser;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;
use textwrap::wrap;

use verifyos_core::engine::Engine;
use verifyos_rules::core::Severity;
use verifyos_rules::entitlements::EntitlementsMismatchRule;
use verifyos_rules::permissions::CameraUsageDescriptionRule;
use verifyos_rules::privacy::MissingPrivacyManifestRule;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the iOS App Bundle (.ipa or .app)
    #[arg(short, long)]
    app: PathBuf,
}

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();

    // 2. Initialize spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .into_diagnostic()?,
    );
    pb.set_message("Analyzing app bundle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // 3. Initialize Core Engine
    let mut engine = Engine::new();

    // Register the current rules
    engine.register_rule(Box::new(MissingPrivacyManifestRule));
    engine.register_rule(Box::new(CameraUsageDescriptionRule));
    engine.register_rule(Box::new(EntitlementsMismatchRule));

    // 4. Run the Engine
    let results = engine
        .run(&args.app)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;

    // 5. Stop the spinner
    pb.finish_with_message("Analysis complete!");

    // 6. Print Report formatting out with comfy-table
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Rule", "Severity", "Result/Message"]);

    let mut has_errors = false;

    for res in results {
        let rule_name = res.rule_name.to_string();

        let severity_cell = match res.severity {
            Severity::Error => Cell::new("ERROR").fg(Color::Red),
            Severity::Warning => Cell::new("WARNING").fg(Color::Yellow),
            Severity::Info => Cell::new("INFO").fg(Color::Blue),
        };

        if let Severity::Error = res.severity {
            if res.result.is_err() {
                has_errors = true;
            }
        }

        let message_cell = match res.result {
            Ok(_) => Cell::new("PASS").fg(Color::Green),
            Err(e) => {
                let err_str = e.to_string();
                let wrapped = wrap(&err_str, 50).join("\n");
                Cell::new(wrapped).fg(Color::Red)
            }
        };

        table.add_row(vec![Cell::new(rule_name), severity_cell, message_cell]);
    }

    println!("\n{}", table);

    // 7. Exit with code 1 if any Error severity check failed
    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}

use std::fs;
use std::path::PathBuf;

fn vscode_file(path: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("editors")
            .join("vscode")
            .join(path),
    )
    .expect("vscode file should be readable")
}

#[test]
fn vscode_extension_launches_voc_lsp() {
    let package_json = vscode_file("package.json");
    let extension_ts = vscode_file("src/extension.ts");

    assert!(package_json.contains("\"version\": \"0.1.3\""));
    assert!(package_json.contains("\"icon\": \"assets/verifyOS_128x.png\""));
    assert!(package_json.contains("\"galleryBanner\""));
    assert!(package_json.contains("\"ai-agent\""));
    assert!(package_json.contains("\"onStartupFinished\""));
    assert!(package_json.contains("\"viewsContainers\""));
    assert!(package_json.contains("\"verifyOS.explorer\""));
    assert!(package_json.contains("\"verifyOS.scanCurrentBundle\""));
    assert!(package_json.contains("\"verifyOS.generateHandoff\""));
    assert!(package_json.contains("\"verifyOS.openProblems\""));
    assert!(package_json.contains("\"verifyOS.restartLanguageServer\""));
    assert!(package_json.contains("\"verifyOS.showOutput\""));
    assert!(package_json.contains("\"verifyOS.path\""));
    assert!(package_json.contains("\"verifyOS.useBundledBinary\""));
    assert!(package_json.contains("\"verifyOS.profile\""));
    assert!(package_json.contains("\"verifyOS.outputDir\""));
    assert!(package_json.contains("\"bin/**\""));
    assert!(package_json.contains("\"CHANGELOG.md\""));
    assert!(extension_ts.contains("[\"lsp\", \"--profile\", profile]"));
    assert!(extension_ts.contains("resolveBundledBinary(context)"));
    assert!(extension_ts.contains("Starting verifyOS language server via"));
    assert!(extension_ts.contains("createTreeView(\"verifyOS.explorer\""));
    assert!(extension_ts.contains("verifyOS: generating handoff bundle"));
    assert!(extension_ts.contains("workbench.actions.view.problems"));
    assert!(extension_ts.contains("verifyOS could not start `voc lsp`"));
}

#[test]
fn vscode_extension_workflow_packages_and_publishes_vsix() {
    let package_json = vscode_file("package.json");
    let changelog = vscode_file("CHANGELOG.md");
    let workflow = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".github")
            .join("workflows")
            .join("vscode-extension.yml"),
    )
    .expect("vscode workflow should be readable");

    assert!(package_json.contains("\"package\": \"vsce package --allow-missing-repository\""));
    assert!(changelog.contains("## 0.1.3"));
    assert!(changelog.contains("Action Center sidebar"));
    assert!(package_json.contains("\"publish:vsce\": \"vsce publish\""));
    assert!(package_json.contains("\"publish:ovsx\": \"ovsx publish\""));
    assert!(package_json.contains("\"LICENSE.md\""));
    assert!(package_json.contains("\"assets/**\""));
    assert!(package_json.contains("\"bin/**\""));
    assert!(workflow.contains("name: VS Code Extension"));
    assert!(workflow.contains("build-bundled-binaries"));
    assert!(workflow.contains("Build bundled binary (${{ matrix.label }})"));
    assert!(workflow.contains("cargo build --release --bin voc"));
    assert!(workflow.contains("vscode-binary-linux-x64"));
    assert!(workflow.contains("vscode-binary-win32-x64"));
    assert!(workflow.contains("vscode-binary-darwin-x64"));
    assert!(workflow.contains("vscode-binary-darwin-arm64"));
    assert!(workflow.contains("npm ci"));
    assert!(workflow.contains("Prepare bundled binary directory"));
    assert!(workflow.contains("npm run compile"));
    assert!(workflow.contains("npm run package -- --out \"$VSIX_NAME\""));
    assert!(workflow.contains("actions/upload-artifact@v4"));
    assert!(workflow.contains("vsce publish --packagePath"));
    assert!(workflow.contains("ovsx publish"));
}

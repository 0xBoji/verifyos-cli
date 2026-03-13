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

    assert!(package_json.contains("\"verifyOS.restartLanguageServer\""));
    assert!(package_json.contains("\"verifyOS.showOutput\""));
    assert!(package_json.contains("\"verifyOS.path\""));
    assert!(package_json.contains("\"verifyOS.profile\""));
    assert!(extension_ts.contains("[\"lsp\", \"--profile\", profile]"));
    assert!(extension_ts.contains("verifyOS could not start `voc lsp`"));
}

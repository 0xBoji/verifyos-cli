use std::fs;
use std::path::{Path, PathBuf};

const NESTED_BUNDLE_CONTAINER_DIRS: &[&str] = &[
    "Frameworks",
    "PlugIns",
    "Extensions",
    "AppClips",
    "Watch",
    "XPCServices",
];
const NESTED_BUNDLE_EXTENSIONS: &[&str] = &["app", "appex", "framework", "xpc"];

#[derive(Debug, thiserror::Error)]
pub enum BundleScanError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct BundleTarget {
    pub bundle_path: PathBuf,
    pub display_name: String,
}

pub fn find_nested_bundles(app_bundle_path: &Path) -> Result<Vec<BundleTarget>, BundleScanError> {
    let mut bundles = Vec::new();

    for dir_name in NESTED_BUNDLE_CONTAINER_DIRS {
        let dir = app_bundle_path.join(dir_name);
        collect_bundles_in_dir(&dir, &mut bundles)?;
    }

    bundles.sort_by(|a, b| a.bundle_path.cmp(&b.bundle_path));
    bundles.dedup_by(|a, b| a.bundle_path == b.bundle_path);

    Ok(bundles)
}

fn collect_bundles_in_dir(
    dir: &Path,
    bundles: &mut Vec<BundleTarget>,
) -> Result<(), BundleScanError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if is_nested_bundle(&path) {
            bundles.push(BundleTarget {
                display_name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                bundle_path: path.clone(),
            });
        }

        if path.is_dir() {
            collect_bundles_in_dir(&path, bundles)?;
        }
    }

    Ok(())
}

fn is_nested_bundle(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            let extension = extension.to_ascii_lowercase();
            NESTED_BUNDLE_EXTENSIONS
                .iter()
                .any(|expected| expected == &extension)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::find_nested_bundles;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn finds_nested_bundles_in_additional_container_directories() {
        let dir = tempdir().expect("temp dir");
        let app_path = dir.path().join("Demo.app");

        std::fs::create_dir_all(app_path.join("Frameworks/Foo.framework"))
            .expect("create framework");
        std::fs::create_dir_all(app_path.join("PlugIns/Share.appex")).expect("create appex");
        std::fs::create_dir_all(app_path.join("Watch/WatchApp.app")).expect("create watch app");
        std::fs::create_dir_all(app_path.join("AppClips/Clip.app")).expect("create app clip");
        std::fs::create_dir_all(app_path.join("XPCServices/Service.xpc")).expect("create xpc");

        let bundles = find_nested_bundles(&app_path).expect("nested bundles");
        let paths: Vec<PathBuf> = bundles
            .into_iter()
            .map(|bundle| {
                bundle
                    .bundle_path
                    .strip_prefix(&app_path)
                    .expect("relative path")
                    .to_path_buf()
            })
            .collect();

        assert!(paths.contains(&Path::new("Frameworks").join("Foo.framework")));
        assert!(paths.contains(&Path::new("PlugIns").join("Share.appex")));
        assert!(paths.contains(&Path::new("Watch").join("WatchApp.app")));
        assert!(paths.contains(&Path::new("AppClips").join("Clip.app")));
        assert!(paths.contains(&Path::new("XPCServices").join("Service.xpc")));
    }
}

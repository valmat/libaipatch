mod support;

use std::path::Path;

use aipatch::engine;
use support::{copy_dir_recursive, snapshot_dir, TempDir};

#[test]
fn upstream_scenarios_match_expected_filesystem_state() -> Result<(), Box<dyn std::error::Error>> {
    let scenarios_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("scenarios");

    for entry in std::fs::read_dir(&scenarios_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let scenario_name = entry.file_name();
        let scenario_name = scenario_name.to_string_lossy();
        if scenario_name.starts_with("011_") || scenario_name.starts_with("015_") {
            // 011 differs intentionally: libaipatch rejects Add File over an existing file.
            // 015 differs intentionally: v1 does not promise full rollback after commit starts.
            continue;
        }

        run_scenario(&path)?;
    }

    Ok(())
}

fn run_scenario(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    copy_dir_recursive(&dir.join("input"), tmp.path())?;

    let patch = std::fs::read_to_string(dir.join("patch.txt"))?;
    let _ = engine::apply(&patch, tmp.path());

    let expected = snapshot_dir(&dir.join("expected"))?;
    let actual = snapshot_dir(tmp.path())?;
    assert_eq!(
        actual,
        expected,
        "scenario {} did not match expected final state",
        dir.display()
    );

    Ok(())
}

//! Integration test that exercises the web build script.
#![allow(clippy::result_large_err)]

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, process::Command};

    // Ensure the web crate compiles for wasm via the build script.
    // This guards against API drift that breaks wasm builds while host tests still pass.
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn web_build_script_succeeds() {
        // Compute repo root from this crate's manifest dir.
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("expected crates/<name> structure")
            .to_path_buf();

        let script = repo_root.join("scripts").join("build_web.py");
        assert!(
            script.exists(),
            "build script not found at {}",
            script.display()
        );

        let output = Command::new("uv")
            .arg("run")
            .arg(&script)
            .current_dir(&repo_root)
            .output()
            .expect("failed to spawn build_web.py");

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "web build failed. status: {}\nstdout:\n{}\nstderr:\n{}",
                output.status, stdout, stderr
            );
        }
    }
}

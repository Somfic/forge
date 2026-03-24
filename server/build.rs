use std::{
    env,
    path::Path,
    process::{Command, Stdio},
};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_dir = Path::new(&manifest_dir).parent().unwrap();
    let frontend_dir = project_dir.join("frontend");

    // Re-run if any frontend source files change
    println!("cargo:rerun-if-changed=../frontend/apps");
    println!("cargo:rerun-if-changed=../modules/movies/frontend/src");

    // Install deps if needed
    let node_modules = frontend_dir.join("node_modules");
    if !node_modules.exists() {
        let status = Command::new("bun")
            .arg("install")
            .current_dir(&frontend_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to run `bun install` — is bun installed?");
        assert!(status.success(), "bun install failed");
    }

    // Build dashboard
    let dashboard_dir = frontend_dir.join("apps/dashboard");
    if dashboard_dir.exists() {
        println!("cargo:warning=Building dashboard frontend...");
        let status = Command::new("bun")
            .args(["run", "build"])
            .current_dir(&dashboard_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to build dashboard frontend");
        assert!(status.success(), "dashboard build failed");
    }

    // Build module frontends
    let modules_dir = project_dir.join("modules");
    if let Ok(entries) = std::fs::read_dir(&modules_dir) {
        for entry in entries.flatten() {
            let fe_dir = entry.path().join("frontend");
            if fe_dir.join("package.json").exists() {
                let module_name = entry.file_name().to_string_lossy().to_string();

                // Generate API client if orval config exists
                if fe_dir.join("orval.config.ts").exists() {
                    println!("cargo:warning=Generating API client for module: {module_name}");
                    let status = Command::new("bunx")
                        .arg("orval")
                        .current_dir(&fe_dir)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .unwrap_or_else(|_| {
                            panic!("failed to generate API client for {module_name}")
                        });
                    if !status.success() {
                        println!(
                            "cargo:warning=API generation failed for {module_name}, skipping (server may not be running)"
                        );
                    }
                }

                println!("cargo:warning=Building frontend for module: {module_name}");
                let status = Command::new("bun")
                    .args(["run", "build"])
                    .current_dir(&fe_dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .unwrap_or_else(|_| panic!("failed to build frontend for {module_name}"));
                assert!(status.success());
            }
        }
    }
}

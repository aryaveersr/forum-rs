use std::process::Command;

fn main() {
    // Get source directory.
    let src_dir = {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        format!("{manifest_dir}/frontend")
    };

    println!("Building the frontend.");
    println!("Source directory: {src_dir}");

    // Execute build command.
    let output = Command::new("pnpm")
        .arg("build")
        .current_dir(src_dir)
        .output()
        .expect("Failed to execute build command.");

    // Ensure that the build succeeded.
    assert!(
        output.status.success(),
        "Build failed. Output:\n{}",
        String::from_utf8(output.stderr).unwrap()
    );

    println!("Frontend built successfully.");

    // Re-run build if these files change.

    // Build script itself.
    build_deps::rerun_if_changed_paths("build.rs").unwrap();

    // Frontend configuration files.
    build_deps::rerun_if_changed_paths("frontend/vite.config.ts").unwrap();
    build_deps::rerun_if_changed_paths("frontend/tsconfig.json").unwrap();
    build_deps::rerun_if_changed_paths("frontend/svelte.config.js").unwrap();
    build_deps::rerun_if_changed_paths("frontend/package.json").unwrap();

    // Frontend source files.
    build_deps::rerun_if_changed_paths("frontend/src/").unwrap();
    build_deps::rerun_if_changed_paths("frontend/src/**/*").unwrap();
}

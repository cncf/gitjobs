use std::{collections::HashMap, fs, path::Path, process::Command};

use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
use which::which;

/// Path to the static assets distribution directory. This path contains a copy
/// of the static assets with some modifications applied (e.g. assets hashed
/// paths).
const STATIC_DIST_PATH: &str = "dist/static";

/// Path to the static assets directory.
const STATIC_PATH: &str = "static";

/// Path to the templates distribution directory. This path contains a copy of
/// the templates with some modifications applied (e.g. assets hashed paths).
const TEMPLATES_DIST_PATH: &str = "dist/templates";

/// Path to the templates directory.
const TEMPLATES_PATH: &str = "templates";

fn main() -> Result<()> {
    // Rerun this build script if changes are detected in the following paths.
    println!("cargo:rerun-if-changed=static");
    println!("cargo:rerun-if-changed=templates");

    // Check if required external tools are available
    if which("tailwindcss").is_err() {
        bail!("tailwindcss not found in PATH (required)");
    }

    // Prepare static assets

    // Build styles using Tailwind CSS
    run(
        "tailwindcss",
        &[
            "-i",
            format!("{STATIC_PATH}/css/styles.src.css").as_str(),
            "-o",
            format!("{STATIC_PATH}/css/styles.css").as_str(),
        ],
    )?;

    // Generate a manifest mapping original asset paths to their hashed versions
    let assets_manifest = generate_static_assets_manifest()?;

    // Copy static assets to the dist directory
    if let Err(err) = fs::remove_dir_all(STATIC_DIST_PATH) {
        if err.kind() != std::io::ErrorKind::NotFound {
            bail!(err);
        }
    }
    copy_dir(Path::new(STATIC_PATH), Path::new(STATIC_DIST_PATH))?;

    // Rename assets files to their hashed versions
    rename_hashed_assets_files(&assets_manifest)?;

    // Replace js assets paths references with their hashed versions
    replace_hashed_assets_refs(&Path::new(STATIC_DIST_PATH).join("js"), &assets_manifest)?;

    // Prepare templates

    // Copy templates to the dist directory
    if let Err(err) = fs::remove_dir_all(TEMPLATES_DIST_PATH) {
        if err.kind() != std::io::ErrorKind::NotFound {
            bail!(err);
        }
    }
    copy_dir(Path::new(TEMPLATES_PATH), Path::new(TEMPLATES_DIST_PATH))?;

    // Replace assets paths references with their hashed versions
    replace_hashed_assets_refs(Path::new(TEMPLATES_DIST_PATH), &assets_manifest)?;

    Ok(())
}

/// Generate static assets manifest. This manifest maps original asset paths to
/// their hashed versions.
fn generate_static_assets_manifest() -> Result<HashMap<String, String>> {
    let mut manifest: HashMap<String, String> = HashMap::new();

    // Add all files in the specified directories to the manifest
    let directories = ["css", "js"];
    for dir in directories {
        let path = Path::new(STATIC_PATH).join(dir);
        add_dir_to_manifest(&path, &mut manifest)?;
    }

    Ok(manifest)
}

/// Add all files in the specified directory and its subdirectories to the
/// assets manifest.
fn add_dir_to_manifest(path: &Path, manifest: &mut HashMap<String, String>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                add_dir_to_manifest(&path, manifest)?;
            } else if path.is_file() {
                add_file_to_manifest(&path, manifest)?;
            }
        }
    }

    Ok(())
}

/// Add a single file to the assets manifest.
fn add_file_to_manifest(path: &Path, manifest: &mut HashMap<String, String>) -> Result<()> {
    // Read file content and calculate its hash
    let content = fs::read(path)?;
    let hash = calculate_hash(&content);

    // Extract file components
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // Prepare hashed file name and path
    let hashed_file_name = if ext.is_empty() {
        format!("{}.{}", stem, &hash[..8])
    } else {
        format!("{}.{}.{}", stem, &hash[..8], ext)
    };
    let hashed_path = path
        .parent()
        .map(|p| p.join(&hashed_file_name))
        .unwrap_or_else(|| Path::new(&hashed_file_name).to_path_buf());

    // Add asset entry to the manifest
    manifest.insert(
        format!("/{}", path.display()),
        format!("/{}", hashed_path.display()),
    );

    Ok(())
}

/// Calculate sha256 hash of content.
fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Helper function to copy a directory recursively.
fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Rename assets files to their hashed versions based on the manifest.
fn rename_hashed_assets_files(manifest: &HashMap<String, String>) -> Result<()> {
    for (plain_path, hashed_path) in manifest {
        let plain_path = Path::new(&format!("{STATIC_DIST_PATH}/"))
            .join(Path::new(plain_path).strip_prefix(format!("/{STATIC_PATH}/"))?);
        let hashed_path = Path::new(&format!("{STATIC_DIST_PATH}/"))
            .join(Path::new(hashed_path).strip_prefix(format!("/{STATIC_PATH}/"))?);

        fs::rename(plain_path, hashed_path)?;
    }

    Ok(())
}

/// Replace assets paths references with their hashed versions in the files in
/// the specified directory and its subdirectories based on the manifest.
fn replace_hashed_assets_refs(path: &Path, manifest: &HashMap<String, String>) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                replace_hashed_assets_refs(&path, manifest)?;
            } else if path.is_file() {
                replace_hashed_assets_refs_in_file(&path, manifest)?;
            }
        }
    }

    Ok(())
}

/// Replace assets paths references with their hashed versions in the specified
/// file based on the manifest.
fn replace_hashed_assets_refs_in_file(path: &Path, manifest: &HashMap<String, String>) -> Result<()> {
    // Read file content
    let original_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::InvalidData {
                bail!(err);
            }
            return Ok(());
        }
    };

    // Replace assets paths with hashed versions
    let mut new_content = original_content.clone();
    for (plain_path, hashed_path) in manifest {
        new_content = new_content.replace(plain_path, hashed_path);
    }

    // Write updated content back to the file if needed
    if new_content != original_content {
        fs::write(path, new_content)?;
    }

    Ok(())
}

/// Helper function to run a command.
fn run(program: &str, args: &[&str]) -> Result<()> {
    // Setup command
    let mut cmd = new_cmd(program);
    cmd.args(args);

    // Execute it and check output
    let output = cmd.output()?;
    if !output.status.success() {
        bail!(
            "\n\n> {cmd:?} (stderr)\n{}\n> {cmd:?} (stdout)\n{}\n",
            String::from_utf8(output.stderr)?,
            String::from_utf8(output.stdout)?
        );
    }

    Ok(())
}

/// Helper function to setup a command based on the target OS.
fn new_cmd(program: &str) -> Command {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", program]);
        cmd
    } else {
        Command::new(program)
    }
}

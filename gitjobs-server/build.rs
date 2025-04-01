use std::process::Command;

use anyhow::{Result, bail};
use which::which;

fn main() -> Result<()> {
    // Rerun this build script if changes are detected in the following paths.
    println!("cargo:rerun-if-changed=static");
    println!("cargo:rerun-if-changed=templates");

    // Check if required external tools are available
    if which("tailwindcss").is_err() {
        bail!("tailwindcss not found in PATH (required)");
    }

    // Build styles
    run(
        "tailwindcss",
        &["-i", "static/css/styles.src.css", "-o", "static/css/styles.css"],
    )?;

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

//! project-private workflow front door. invoke via `cargo x <subcommand>`
//! (alias defined in `.cargo/config.toml`)
//!
//! subcommands:
//! - `update-schema`: runs `example-mycel`, writes its output to
//!   `example-mycel/schema.json`. used after editing the `Config` struct
//!   to refresh the checked-in reference that the drift check diffs against

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(subcommand) = args.next() else {
        eprintln!("usage: cargo x <subcommand>");
        eprintln!();
        eprintln!("subcommands:");
        eprintln!("    update-schema    re-emit example-mycel/schema.json");
        return ExitCode::FAILURE;
    };

    match subcommand.as_str() {
        "update-schema" => update_schema(),
        other => {
            eprintln!("unknown subcommand: {other}");
            ExitCode::FAILURE
        }
    }
}

fn update_schema() -> ExitCode {
    let workspace_root = workspace_root();
    let target = workspace_root.join("example-mycel").join("schema.json");

    let output = Command::new("cargo")
        .args(["run", "--quiet", "-p", "example-mycel"])
        .current_dir(&workspace_root)
        .output()
        .expect("failed to run cargo");

    if !output.status.success() {
        eprintln!(
            "example-mycel failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return ExitCode::FAILURE;
    }

    fs::write(&target, &output.stdout).expect("failed to write schema.json");
    println!("wrote {}", target.display());
    ExitCode::SUCCESS
}

fn workspace_root() -> PathBuf {
    // xtask lives at <root>/xtask; walk up one from its Cargo.toml
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live in a workspace")
        .to_path_buf()
}

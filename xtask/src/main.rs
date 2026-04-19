//! project-private workflow front door. invoke via `cargo x <subcommand>`
//! (alias defined in `.cargo/config.toml`)
//!
//! subcommands:
//! - `update-schema`: runs `example-mycel`, writes its output to
//!   `example-mycel/schema.json`. used after editing the `Config` struct
//!   to refresh the checked-in reference that the drift check diffs against
//! - `check-semver`: runs `cargo semver-checks` on the publishable
//!   workspace members (`nixcfg`, `nixcfg-derive`). requires a published
//!   baseline to diff against, so only meaningful once the crates have
//!   shipped a first release

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(subcommand) = args.next() else {
        print_usage();
        return ExitCode::FAILURE;
    };

    match subcommand.as_str() {
        "update-schema" => update_schema(),
        "check-semver" => check_semver(args.collect::<Vec<_>>()),
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown subcommand: {other}");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("usage: cargo x <subcommand> [args...]");
    eprintln!();
    eprintln!("subcommands:");
    eprintln!("    update-schema    re-emit example-mycel/schema.json");
    eprintln!("    check-semver     run cargo-semver-checks on nixcfg + nixcfg-derive");
    eprintln!();
    eprintln!("extra args to check-semver are passed through to cargo-semver-checks:");
    eprintln!("    cargo x check-semver --baseline-rev v0.3.0");
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

fn check_semver(extra_args: Vec<String>) -> ExitCode {
    let workspace_root = workspace_root();

    let status = Command::new("cargo")
        .args([
            "semver-checks",
            "check-release",
            "--workspace",
            // publish = false crates (example-mycel, xtask) don't go to
            // crates.io, so they have no baseline to diff against
            "--exclude",
            "example-mycel",
            "--exclude",
            "xtask",
        ])
        .args(&extra_args)
        .current_dir(&workspace_root)
        .status()
        .expect("failed to run cargo semver-checks (is it installed?)");

    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn workspace_root() -> PathBuf {
    // xtask lives at <root>/xtask; walk up one from its Cargo.toml
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live in a workspace")
        .to_path_buf()
}

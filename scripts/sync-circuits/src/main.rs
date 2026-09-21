//! CLI utility to compile Noir circuits and sync JSON artifacts.
//!
//! Runs `nargo compile` for each circuit, then copies the produced
//! JSON files to the services that need them:
//! - `services/merkle/circuits/`
//! - `web/public/circuits/`
//!
//! Usage (inside container):
//!   cd /home/ubuntu
//!   cargo run --manifest-path scripts/sync-circuits/Cargo.toml

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Circuits to compile and sync.
/// (name, source_dir, target_json_filename)
const CIRCUITS: &[(&str, &str, &str)] = &[
    ("hash2", "circuits/hash2", "hash2.json"),
    ("hashes", "circuits/hashes", "hashes.json"),
    ("withdrawal", "circuits/withdrawal", "withdrawal.json"),
];

/// Destinations to copy the JSON files to.
const DESTINATIONS: &[&str] = &["services/merkle/circuits", "web/public/circuits"];

fn main() -> Result<()> {
    let project_root = find_project_root()?;
    println!("📁 Project root: {}", project_root.display());

    // Compile all circuits
    for (name, src_dir, _) in CIRCUITS {
        compile_circuit(name, &project_root.join(src_dir))?;
    }

    // Sync JSON artifacts
    for dest in DESTINATIONS {
        let dest_dir = project_root.join(dest);
        sync_to_destination(&project_root, &dest_dir)?;
    }

    println!("✅ Done.");
    Ok(())
}

/// Finds the project root (directory containing `circuits/` and `onchain/`).
fn find_project_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current dir")?;

    loop {
        if dir.join("circuits").is_dir() && dir.join("onchain").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            anyhow::bail!(
                "Could not find project root (directory with `circuits/` and `onchain/`)"
            );
        }
    }
}

/// Runs `nargo compile` in the given circuit directory.
fn compile_circuit(name: &str, circuit_dir: &Path) -> Result<()> {
    println!("🔨 Compiling circuit: {}", name);
    println!("   dir: {}", circuit_dir.display());

    let status = Command::new("nargo")
        .arg("compile")
        .current_dir(circuit_dir)
        .status()
        .with_context(|| format!("Failed to run `nargo compile` in {}", circuit_dir.display()))?;

    if !status.success() {
        anyhow::bail!(
            "`nargo compile` failed for {} (exit code: {:?})",
            name,
            status.code()
        );
    }

    Ok(())
}

/// Copies JSON artifacts from `circuits/*/target/` to the destination directory.
fn sync_to_destination(project_root: &Path, dest_dir: &Path) -> Result<()> {
    println!("📦 Syncing to: {}", dest_dir.display());
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create {}", dest_dir.display()))?;

    for (_, src_dir, target_json) in CIRCUITS {
        let src = project_root.join(src_dir).join("target").join(target_json);
        let dst = dest_dir.join(target_json);

        if !src.exists() {
            anyhow::bail!(
                "Source JSON not found: {}. Did `nargo compile` run?",
                src.display()
            );
        }

        fs::copy(&src, &dst)
            .with_context(|| format!("Failed to copy {} → {}", src.display(), dst.display()))?;

        println!("   ✓ {}", target_json);
    }

    Ok(())
}

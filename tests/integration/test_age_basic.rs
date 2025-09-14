//! Basic Cage Integration Test
//! Validates that cage CLI works correctly with Age

use std::process::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cage_cli_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 BASIC CAGE CLI TEST");
    println!("======================");

    // Test that cage demo command works
    let output = Command::new("cargo")
        .args(&["run", "--", "demo"])
        .output()?;

    assert!(output.status.success(), "Cage demo command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cage - Age Encryption Demonstration"));

    println!("✅ Cage demo command works");
    Ok(())
}

#[test]
fn test_cage_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 CAGE CLI HELP TEST");
    println!("=====================");

    // Test that cage help works
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()?;

    assert!(output.status.success(), "Cage help command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Age encryption automation"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("lock"));
    assert!(stdout.contains("unlock"));

    println!("✅ Cage help command works");
    Ok(())
}

#[test]
fn test_cage_cli_version() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 CAGE CLI VERSION TEST");
    println!("========================");

    // Test that cage version works
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()?;

    assert!(output.status.success(), "Cage version command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cage 0.1.0"));

    println!("✅ Cage version command works");
    Ok(())
}
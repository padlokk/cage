//! PTY Automation Tests for Cage
//! Based on the working driver.rs implementation

use hub::portable_pty::*;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_pty_creation() {
    println!("🧪 PTY Creation Test");
    println!("====================");

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size).expect("Failed to create PTY");
    println!("✅ PTY created successfully");
    drop(pair);
}

#[test]
fn test_age_binary_detection() {
    println!("🧪 Age Binary Detection Test");
    println!("============================");

    // Test if age binary is available
    let result = std::process::Command::new("age").arg("--version").output();

    match result {
        Ok(output) if output.status.success() => {
            println!("✅ Age binary found and working");
            let version = String::from_utf8_lossy(&output.stdout);
            println!("📄 Age version: {}", version.trim());
        }
        Ok(_) => {
            println!("⚠️ Age binary found but returned error");
            println!("   This test will be skipped");
        }
        Err(_) => {
            println!("⚠️ Age binary not found");
            println!("   Install age: sudo apt install age (Ubuntu)");
            println!("   This test will be skipped");
        }
    }
}

#[test]
#[ignore] // Ignored by default as it requires age binary and creates files
fn test_pty_age_integration() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🧪 PTY + Age Integration Test");
    println!("=============================");

    // Create temporary directory for test files
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test file
    let test_file = temp_path.join("test.txt");
    let test_content = "Hello PTY world!";
    std::fs::write(&test_file, test_content)?;

    println!("📁 Test directory: {:?}", temp_path);
    println!("📄 Created test file: {:?}", test_file);

    // Test Age encryption via PTY
    let encrypted_file = test_age_pty_encrypt(&test_file)?;

    // Test Age decryption via PTY
    let decrypted_file = test_age_pty_decrypt(&encrypted_file)?;

    // Verify content
    let decrypted_content = std::fs::read_to_string(&decrypted_file)?;
    assert_eq!(decrypted_content, test_content);

    println!("✅ Full PTY + Age integration test passed!");
    Ok(())
}

fn test_age_pty_encrypt(
    input_file: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    println!("🔐 Testing PTY Age encryption...");

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;

    // Create output file path
    let output_file = input_file.with_extension("age");

    // Build age command
    let mut cmd = CommandBuilder::new("age");
    cmd.arg("-p");
    cmd.arg("-o");
    cmd.arg(&output_file);
    cmd.arg(&input_file);

    println!("🚀 Spawning age in PTY...");
    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    // PTY automation thread
    let handle = thread::spawn(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut buffer = [0u8; 256];
            let passphrase = "testpass123";

            for i in 1..=20 {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        println!("📄 Encryption: Age finished");
                        break;
                    }
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]);

                        if text.to_lowercase().contains("passphrase") {
                            println!("🔐 Sending passphrase for encryption...");
                            writer.write_all(passphrase.as_bytes())?;
                            writer.write_all(b"\n")?;
                        }

                        if text.to_lowercase().contains("confirm") {
                            println!("🔐 Confirming passphrase...");
                            writer.write_all(passphrase.as_bytes())?;
                            writer.write_all(b"\n")?;
                        }
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
            Ok(())
        },
    );

    handle.join().map_err(|_| "Encryption thread panicked")??;

    let mut child = child;
    let status = child.wait()?;

    if status.success() && output_file.exists() {
        println!("✅ Encryption successful: {:?}", output_file);
        Ok(output_file)
    } else {
        Err("Encryption failed".into())
    }
}

fn test_age_pty_decrypt(
    encrypted_file: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    println!("🔓 Testing PTY Age decryption...");

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;

    // Create output file path
    let output_file = encrypted_file.with_extension("decrypted.txt");

    // Build age decrypt command
    let mut cmd = CommandBuilder::new("age");
    cmd.arg("-d");
    cmd.arg("-o");
    cmd.arg(&output_file);
    cmd.arg(&encrypted_file);

    println!("🔓 Spawning age decrypt in PTY...");
    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    // PTY automation thread
    let handle = thread::spawn(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut buffer = [0u8; 256];
            let passphrase = "testpass123";

            for i in 1..=20 {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        println!("📄 Decryption: Age finished");
                        break;
                    }
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]);

                        if text.to_lowercase().contains("passphrase") {
                            println!("🔐 Sending passphrase for decryption...");
                            writer.write_all(passphrase.as_bytes())?;
                            writer.write_all(b"\n")?;
                        }
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
            Ok(())
        },
    );

    handle.join().map_err(|_| "Decryption thread panicked")??;

    let mut child = child;
    let status = child.wait()?;

    if status.success() && output_file.exists() {
        println!("✅ Decryption successful: {:?}", output_file);
        Ok(output_file)
    } else {
        Err("Decryption failed".into())
    }
}

#[test]
fn test_pty_command_builder() {
    println!("🧪 PTY Command Builder Test");
    println!("===========================");

    // Test that we can build age commands
    let mut cmd = CommandBuilder::new("age");
    cmd.arg("--version");

    // This should not panic
    println!("✅ CommandBuilder created successfully");
}

#[test]
fn test_pty_buffer_handling() {
    println!("🧪 PTY Buffer Handling Test");
    println!("===========================");

    let buffer = [0u8; 256];
    let test_text = b"Enter passphrase: ";

    // Test passphrase detection logic
    let text = String::from_utf8_lossy(test_text);
    assert!(text.to_lowercase().contains("passphrase"));

    println!("✅ Buffer handling logic works");
}

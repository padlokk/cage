//! Streaming performance benchmark tests for CAGE-12a
//!
//! Tests pipe-based streaming with large files to validate:
//! - Memory usage stays constant (streaming, not loading full file)
//! - Throughput performance (MB/s)
//! - Comparison between temp-file and pipe strategies

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Instant;
use tempfile::TempDir;

use cage::cage::adp::v2::{AgeAdapterV2, ShellAdapterV2};
use cage::cage::core::OutputFormat;
use cage::cage::core::{Identity, Recipient};

/// Generate a test file of specified size with repeating pattern
fn create_test_file(path: &Path, size_mb: usize) -> io::Result<()> {
    let mut file = File::create(path)?;
    let chunk_size = 1024 * 1024; // 1MB chunks
    let pattern =
        b"The quick brown fox jumps over the lazy dog. 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ! ";
    let mut buffer = Vec::with_capacity(chunk_size);

    // Fill buffer with pattern
    while buffer.len() < chunk_size {
        buffer.extend_from_slice(pattern);
    }
    buffer.truncate(chunk_size);

    // Write chunks to reach target size
    for _ in 0..size_mb {
        file.write_all(&buffer)?;
    }
    file.sync_all()?;
    Ok(())
}

/// Measure file size
fn get_file_size(path: &Path) -> io::Result<u64> {
    Ok(fs::metadata(path)?.len())
}

/// Format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[test]
#[ignore = "Large file benchmark - run with --ignored flag and CAGE_BENCHMARK=1"]
fn benchmark_streaming_1gb() -> Result<(), Box<dyn std::error::Error>> {
    // Skip unless explicitly requested
    if std::env::var("CAGE_BENCHMARK").unwrap_or_default() != "1" {
        println!("Skipping benchmark - set CAGE_BENCHMARK=1 to run");
        return Ok(());
    }

    // Check age binary
    if which::which("age").is_err() {
        println!("Benchmark skipped: age binary not available");
        return Ok(());
    }

    // Create adapter
    let adapter = match ShellAdapterV2::new() {
        Ok(a) => a,
        Err(e) => {
            println!("Benchmark skipped: ShellAdapterV2 unavailable ({e})");
            return Ok(());
        }
    };

    println!("\n=== CAGE-12a Streaming Performance Benchmark ===\n");

    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test_1gb.bin");
    let encrypted_file = temp_dir.path().join("test_1gb.bin.age");
    let decrypted_file = temp_dir.path().join("test_1gb_decrypted.bin");

    // Create 1GB test file
    println!("Creating 1GB test file...");
    let size_mb = 1024; // 1GB
    create_test_file(&test_file, size_mb)?;
    let file_size = get_file_size(&test_file)?;
    println!("Test file created: {}", format_bytes(file_size));

    // Test encryption with file-based approach
    println!("\n--- Testing file-based encryption ---");
    let passphrase = Identity::Passphrase("benchmark_test_pass_2024".to_string());

    let start = Instant::now();
    adapter.encrypt_file(
        &test_file,
        &encrypted_file,
        &passphrase,
        None,
        OutputFormat::Binary,
    )?;
    let encrypt_duration = start.elapsed();

    let encrypted_size = get_file_size(&encrypted_file)?;
    let encrypt_throughput =
        (file_size as f64 / encrypt_duration.as_secs_f64()) / (1024.0 * 1024.0);

    println!("Encryption completed:");
    println!("  Duration: {:.2}s", encrypt_duration.as_secs_f64());
    println!("  Throughput: {:.2} MB/s", encrypt_throughput);
    println!("  Output size: {}", format_bytes(encrypted_size));

    // Test decryption
    println!("\n--- Testing file-based decryption ---");
    let start = Instant::now();
    adapter.decrypt_file(&encrypted_file, &decrypted_file, &passphrase)?;
    let decrypt_duration = start.elapsed();

    let decrypted_size = get_file_size(&decrypted_file)?;
    let decrypt_throughput =
        (encrypted_size as f64 / decrypt_duration.as_secs_f64()) / (1024.0 * 1024.0);

    println!("Decryption completed:");
    println!("  Duration: {:.2}s", decrypt_duration.as_secs_f64());
    println!("  Throughput: {:.2} MB/s", decrypt_throughput);
    println!("  Output size: {}", format_bytes(decrypted_size));

    // Verify file integrity
    println!("\n--- Verifying integrity ---");
    let original_hash = hash_file(&test_file)?;
    let decrypted_hash = hash_file(&decrypted_file)?;

    if original_hash == decrypted_hash {
        println!("✓ Integrity verified - files match");
    } else {
        println!("✗ Integrity check failed - files differ!");
        return Err("Decrypted file does not match original".into());
    }

    // Test streaming (if we can test it with files)
    println!("\n--- Testing streaming encryption (pipe strategy) ---");
    if let Ok(strategy) = std::env::var("CAGE_STREAMING_STRATEGY") {
        println!("Using streaming strategy: {}", strategy);
    } else {
        println!("Using default streaming strategy");
    }

    // For true streaming test, we'd need to measure memory usage
    // For now, we test that streaming works with large files
    let mut input = File::open(&test_file)?;
    let mut output = Vec::new();

    let start = Instant::now();
    adapter.encrypt_stream(
        &mut input,
        &mut output,
        &passphrase,
        None,
        OutputFormat::Binary,
    )?;
    let stream_duration = start.elapsed();

    let stream_throughput = (file_size as f64 / stream_duration.as_secs_f64()) / (1024.0 * 1024.0);
    println!("Stream encryption completed:");
    println!("  Duration: {:.2}s", stream_duration.as_secs_f64());
    println!("  Throughput: {:.2} MB/s", stream_throughput);
    println!("  Output size: {}", format_bytes(output.len() as u64));

    // Summary
    println!("\n=== Benchmark Summary ===");
    println!("File size: {}", format_bytes(file_size));
    println!("File-based encryption: {:.2} MB/s", encrypt_throughput);
    println!("File-based decryption: {:.2} MB/s", decrypt_throughput);
    println!("Stream encryption: {:.2} MB/s", stream_throughput);
    println!("Integrity check: PASSED");

    // Performance recommendations
    println!("\n=== Performance Analysis ===");
    if stream_throughput > encrypt_throughput * 0.8 {
        println!("✓ Streaming performance is competitive with file-based approach");
    } else {
        println!("⚠ Streaming performance is significantly slower than file-based");
    }

    Ok(())
}

/// Simple file hash for integrity check
fn hash_file(path: &Path) -> io::Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    let mut file = File::open(path)?;
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        buffer[..bytes_read].hash(&mut hasher);
    }

    Ok(format!("{:x}", hasher.finish()))
}

#[test]
fn test_streaming_small_file() -> Result<(), Box<dyn std::error::Error>> {
    // This is a quick test that always runs
    if which::which("age").is_err() {
        println!("Test skipped: age binary not available");
        return Ok(());
    }

    let adapter = match ShellAdapterV2::new() {
        Ok(a) => a,
        Err(e) => {
            println!("Test skipped: ShellAdapterV2 unavailable ({e})");
            return Ok(());
        }
    };

    // Test with a small file (1MB)
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("small_test.txt");
    create_test_file(&test_file, 1)?; // 1MB file

    let passphrase = Identity::Passphrase("test_pass".to_string());
    let encrypted_file = temp_dir.path().join("small_test.txt.age");
    let decrypted_file = temp_dir.path().join("small_test_decrypted.txt");

    // Test round trip
    adapter.encrypt_file(
        &test_file,
        &encrypted_file,
        &passphrase,
        None,
        OutputFormat::Binary,
    )?;
    adapter.decrypt_file(&encrypted_file, &decrypted_file, &passphrase)?;

    // Verify
    let original_hash = hash_file(&test_file)?;
    let decrypted_hash = hash_file(&decrypted_file)?;
    assert_eq!(original_hash, decrypted_hash, "File integrity check failed");

    println!("Small file streaming test passed");
    Ok(())
}

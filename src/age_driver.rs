use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use portable_pty::*;

fn main() {
    println!("🧪 Simple PTY + age driver test");

    // Show current directory
    println!("📁 Current directory: {:?}", std::env::current_dir().unwrap());

    // Create test file
    std::fs::write("test.txt", "Hello PTY world!").expect("Failed to create test file");

    // Verify it exists
    if std::path::Path::new("test.txt").exists() {
        println!("✅ Test file created");
    } else {
        println!("❌ Test file creation failed");
        return;
    }

    match test_age_pty() {
        Ok(_) => println!("✅ Success!"),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // Cleanup disabled - files left for verification
    // let _ = std::fs::remove_file("test.txt");
    // let _ = std::fs::remove_file("test.age");
}

fn test_age_pty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📝 Creating PTY...");
    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;
    println!("✅ PTY created");

    // Get absolute paths
    let current_dir = std::env::current_dir()?;
    let input_path = current_dir.join("test.txt");
    let output_path = current_dir.join("test.age");

    println!("📄 Input: {:?}", input_path);
    println!("📄 Output: {:?}", output_path);

    // Build age command with absolute paths
    let mut cmd = CommandBuilder::new("age");
    cmd.arg("-p");
    cmd.arg("-o");
    cmd.arg(&output_path);
    cmd.arg(&input_path);

    println!("🚀 Spawning age in PTY...");
    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);  // Important: close slave in parent

    let mut writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    println!("🔄 Starting automation thread...");
    let handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0u8; 256];
        let passphrase = "testpass123";

        println!("👂 Thread: Listening for age output...");

        for i in 1..=20 {  // Try 20 times, 100ms each = 2 seconds max
            println!("👂 Thread: Read attempt {}", i);

            match reader.read(&mut buffer) {
                Ok(0) => {
                    println!("📄 Thread: EOF - age finished");
                    break;
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]);
                    println!("📨 Thread: Got output: {:?}", text);

                    // Look for passphrase prompt
                    if text.to_lowercase().contains("passphrase") {
                        println!("🔐 Thread: Sending passphrase...");
                        writer.write_all(passphrase.as_bytes())?;
                        writer.write_all(b"\n")?;
                        println!("✅ Thread: Passphrase sent");
                    }

                    if text.to_lowercase().contains("confirm") {
                        println!("🔐 Thread: Sending confirmation...");
                        writer.write_all(passphrase.as_bytes())?;
                        writer.write_all(b"\n")?;
                        println!("✅ Thread: Confirmation sent");
                    }
                }
                Err(e) => {
                    println!("⚠️ Thread: Read error (attempt {}): {}", i, e);
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("🏁 Thread: Finished");
        Ok(())
    });

    println!("⏰ Main: Waiting for automation thread...");
    handle.join().map_err(|_| "Thread panicked")??;

    println!("⏰ Main: Waiting for age process...");
    let mut child = child;
    let status = child.wait()?;
    println!("🏁 Main: Age exited with status: {:?}", status);

    let output_path = std::env::current_dir()?.join("test.age");
    if output_path.exists() {
        println!("✅ Encrypted file created!");

        // Now test decryption to complete UAT
        println!("🔓 Testing decryption...");
        let decrypt_result = test_decrypt(&output_path);
        match decrypt_result {
            Ok(_) => println!("✅ Full encrypt/decrypt cycle successful!"),
            Err(e) => {
                println!("❌ Decryption failed: {}", e);
                return Err("Full UAT cycle failed".into());
            }
        }

        return Ok(());
    } else {
        println!("❌ Encrypted file not found");
        return Err("Encryption failed".into());
    }
}

fn test_decrypt(encrypted_file: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use portable_pty::*;

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;

    let current_dir = std::env::current_dir()?;
    let output_path = current_dir.join("test_decrypted.txt");

    println!("📄 Decrypt input: {:?}", encrypted_file);
    println!("📄 Decrypt output: {:?}", output_path);

    // Build age decrypt command
    let mut cmd = CommandBuilder::new("age");
    cmd.arg("-d");
    cmd.arg("-o");
    cmd.arg(&output_path);
    cmd.arg(encrypted_file);

    println!("🔓 Spawning age decrypt in PTY...");
    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    println!("🔄 Starting decryption automation thread...");
    let handle = thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0u8; 256];
        let passphrase = "testpass123";

        println!("👂 Decrypt thread: Listening for age output...");

        for i in 1..=20 {
            println!("👂 Decrypt thread: Read attempt {}", i);

            match reader.read(&mut buffer) {
                Ok(0) => {
                    println!("📄 Decrypt thread: EOF - age finished");
                    break;
                }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]);
                    println!("📨 Decrypt thread: Got output: {:?}", text);

                    if text.to_lowercase().contains("passphrase") {
                        println!("🔐 Decrypt thread: Sending passphrase...");
                        writer.write_all(passphrase.as_bytes())?;
                        writer.write_all(b"\n")?;
                        println!("✅ Decrypt thread: Passphrase sent");
                    }
                }
                Err(e) => {
                    println!("⚠️ Decrypt thread: Read error (attempt {}): {}", i, e);
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("🏁 Decrypt thread: Finished");
        Ok(())
    });

    println!("⏰ Main: Waiting for decrypt automation thread...");
    handle.join().map_err(|_| "Decrypt thread panicked")??;

    println!("⏰ Main: Waiting for decrypt age process...");
    let mut child = child;
    let status = child.wait()?;
    println!("🏁 Main: Age decrypt exited with status: {:?}", status);

    if output_path.exists() {
        let decrypted_content = std::fs::read_to_string(&output_path)?;
        println!("✅ Decrypted file created!");
        println!("📄 Decrypted content: {:?}", decrypted_content);

        // Verify content matches original
        if decrypted_content == "Hello PTY world!" {
            println!("✅ Content verification passed!");
            return Ok(());
        } else {
            return Err(format!("Content mismatch: expected 'Hello PTY world!', got '{}'", decrypted_content).into());
        }
    } else {
        return Err("Decrypted file not found".into());
    }
}
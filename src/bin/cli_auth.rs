//! Authority Chain Direct CLI Interface
//!
//! This CLI provides direct access to the X->M->R->I->D authority chain module system for 
//! testing and debugging. Users can directly interface with authority key generation,
//! authority-based encryption operations, ignition key workflows, and chain validation
//! without going through the main padlock orchestrator.
//!
//! Security Guardian: Edgar - Direct authority chain interface

use std::path::{Path, PathBuf};
use std::process;
use clap::{Parser, Subcommand};
use std::fs;

// Import our authority modules
use padlock::auth::{
    AuthorityChain, KeyType,
    operations::{AuthorityAgeKeyGenerator, AuthorityAgeEncryption, EncryptionParams},
    ignition::IgnitionKey,
};
use padlock::sec::cage::config::OutputFormat;

/// Authority Chain Direct CLI Interface
#[derive(Parser)]
#[command(name = "cli_auth")]
#[command(about = "Direct interface to X->M->R->I->D authority chain system")]
#[command(version = "0.0.1-auth-cli")]
#[command(author = "Edgar (Security Guardian)")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Keys directory for authority chain operations
    #[arg(long, default_value = "./keys")]
    keys_dir: PathBuf,
    
    /// Output format for encrypted files
    #[arg(long, default_value = "binary")]
    format: OutputFormatArg,
    
    #[command(subcommand)]
    command: Commands,
}

/// Authority chain commands for testing and debugging
#[derive(Subcommand)]
enum Commands {
    /// Generate complete X->M->R->I->D authority chain
    Generate {
        /// Base name for the authority chain
        #[arg(short, long, default_value = "auth")]
        name: String,
        
        /// Output directory for generated keys
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
    
    /// Encrypt file using authority key
    Encrypt {
        /// Input file to encrypt
        input_file: PathBuf,
        
        /// Authority level to use for encryption
        #[arg(short, long)]
        authority_level: AuthorityLevelArg,
        
        /// Output file (optional, will add .age suffix if not provided)
        #[arg(short, long)]
        output_file: Option<PathBuf>,
        
        /// Authority key fingerprint (optional, will find by level)
        #[arg(long)]
        key_fingerprint: Option<String>,
        
        /// Verify authority before encrypting
        #[arg(long, default_value = "true")]
        verify_authority: bool,
    },
    
    /// Decrypt file using authority key
    Decrypt {
        /// Encrypted input file
        input_file: PathBuf,
        
        /// Output file for decrypted content
        output_file: PathBuf,
        
        /// Authority level to use for decryption
        #[arg(short, long)]
        authority_level: AuthorityLevelArg,
        
        /// Authority key fingerprint (optional, will find by level)
        #[arg(long)]
        key_fingerprint: Option<String>,
    },
    
    /// Create ignition key with passphrase protection
    IgnitionCreate {
        /// Authority level for ignition key (ignition or distro)
        #[arg(short, long, default_value = "ignition")]
        authority_level: AuthorityLevelArg,
        
        /// Passphrase for ignition key protection
        #[arg(short, long)]
        passphrase: String,
        
        /// Optional name for the ignition key
        #[arg(short, long)]
        name: Option<String>,
    },
    
    /// Encrypt file using ignition key with passphrase
    IgnitionEncrypt {
        /// Input file to encrypt
        input_file: PathBuf,
        
        /// Ignition key file
        #[arg(short, long)]
        key_file: PathBuf,
        
        /// Passphrase for unlocking ignition key
        #[arg(short, long)]
        passphrase: String,
        
        /// Output file (optional, will add .age suffix if not provided)
        #[arg(short, long)]
        output_file: Option<PathBuf>,
    },
    
    /// Validate authority relationships in the chain
    Validate {
        /// Test all authority relationships
        #[arg(long)]
        test_all: bool,
        
        /// Test specific authority relationship (format: parent:child)
        #[arg(long)]
        test_pair: Option<String>,
        
        /// Show detailed validation results
        #[arg(long)]
        detailed: bool,
    },
    
    /// Show status of authority chain and keys
    Status {
        /// Show complete chain information
        #[arg(long)]
        show_chain: bool,
        
        /// Show individual key details
        #[arg(long)]
        show_keys: bool,
        
        /// Show authority relationships
        #[arg(long)]
        show_authorities: bool,
        
        /// Chain name to look for (matches generate --name)
        #[arg(short, long, default_value = "auth")]
        name: String,
    },
    
    /// Test complete end-to-end workflow
    Test {
        /// Run full workflow test
        #[arg(long)]
        full_workflow: bool,
        
        /// Test specific authority level
        #[arg(long)]
        test_level: Option<AuthorityLevelArg>,
        
        /// Run performance benchmarks
        #[arg(long)]
        benchmark: bool,
    },
    
    /// Run demonstration of authority chain capabilities
    Demo {
        /// Demo scenario to run
        #[arg(default_value = "basic")]
        scenario: DemoScenario,
        
        /// Clean up demo files after completion
        #[arg(long, default_value = "true")]
        cleanup: bool,
    },
}

/// Authority levels for CLI argument parsing
#[derive(Clone, Debug, clap::ValueEnum)]
enum AuthorityLevelArg {
    Skull,
    Master, 
    Repo,
    Ignition,
    Distro,
}

impl From<AuthorityLevelArg> for KeyType {
    fn from(level: AuthorityLevelArg) -> Self {
        match level {
            AuthorityLevelArg::Skull => KeyType::Skull,
            AuthorityLevelArg::Master => KeyType::Master,
            AuthorityLevelArg::Repo => KeyType::Repo,
            AuthorityLevelArg::Ignition => KeyType::Ignition,
            AuthorityLevelArg::Distro => KeyType::Distro,
        }
    }
}

/// Output format for CLI argument parsing
#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormatArg {
    Binary,
    Ascii,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(format: OutputFormatArg) -> Self {
        match format {
            OutputFormatArg::Binary => OutputFormat::Binary,
            OutputFormatArg::Ascii => OutputFormat::AsciiArmor,
        }
    }
}

/// Demo scenarios for authority chain testing
#[derive(Clone, Debug, clap::ValueEnum)]
enum DemoScenario {
    Basic,
    FullChain,
    Ignition,
    Validation,
    Performance,
}

/// CLI Application State
struct CliAuth {
    verbose: bool,
    keys_dir: PathBuf,
    format: OutputFormat,
}

impl CliAuth {
    fn new(verbose: bool, keys_dir: PathBuf, format: OutputFormat) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure keys directory exists
        fs::create_dir_all(&keys_dir)?;
        
        Ok(Self {
            verbose,
            keys_dir,
            format,
        })
    }
    
    fn run(&mut self, command: Commands) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Generate { name, output_dir } => {
                self.handle_generate(name, output_dir)
            }
            Commands::Encrypt { input_file, authority_level, output_file, key_fingerprint, verify_authority } => {
                self.handle_encrypt(input_file, authority_level, output_file, key_fingerprint, verify_authority)
            }
            Commands::Decrypt { input_file, output_file, authority_level, key_fingerprint } => {
                self.handle_decrypt(input_file, output_file, authority_level, key_fingerprint)
            }
            Commands::IgnitionCreate { authority_level, passphrase, name } => {
                self.handle_ignition_create(authority_level, passphrase, name)
            }
            Commands::IgnitionEncrypt { input_file, key_file, passphrase, output_file } => {
                self.handle_ignition_encrypt(input_file, key_file, passphrase, output_file)
            }
            Commands::Validate { test_all, test_pair, detailed } => {
                self.handle_validate(test_all, test_pair, detailed)
            }
            Commands::Status { show_chain, show_keys, show_authorities, name } => {
                self.handle_status(show_chain, show_keys, show_authorities, name)
            }
            Commands::Test { full_workflow, test_level, benchmark } => {
                self.handle_test(full_workflow, test_level, benchmark)
            }
            Commands::Demo { scenario, cleanup } => {
                self.handle_demo(scenario, cleanup)
            }
        }
    }
    
    fn handle_generate(&self, name: String, output_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔑 Generating X->M->R->I->D Authority Chain");
        println!("==========================================");
        
        let output_path = output_dir.unwrap_or_else(|| self.keys_dir.clone());
        fs::create_dir_all(&output_path)?;
        
        println!("📁 Output Directory: {}", output_path.display());
        println!("🏷️  Chain Name: {}", name);
        
        // Create authority chain and key generator
        let authority_chain = AuthorityChain::new();
        let mut key_generator = AuthorityAgeKeyGenerator::new(authority_chain, None)?;
        
        // Generate complete authority chain
        let generated_keys = key_generator.generate_complete_authority_chain(&name, &output_path)?;
        
        println!("✅ Generated {} authority keys:", generated_keys.len());
        for (i, key) in generated_keys.iter().enumerate() {
            println!("   {}. {} - {}", 
                i + 1, 
                key.authority_key.key_type(), 
                key.age_public_key
            );
            
            if let Some(path) = &key.key_file_path {
                println!("      📄 Key file: {}", path.display());
            }
        }
        
        println!("\n🎉 Authority chain generation completed successfully!");
        Ok(())
    }
    
    fn handle_encrypt(&self, input_file: PathBuf, authority_level: AuthorityLevelArg, output_file: Option<PathBuf>, _key_fingerprint: Option<String>, verify_authority: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔐 Authority-Based File Encryption");
        println!("==================================");
        
        let output_path = output_file.unwrap_or_else(|| {
            input_file.with_extension(format!("{}.age", 
                input_file.extension().and_then(|s| s.to_str()).unwrap_or("txt")
            ))
        });
        
        println!("📄 Input File: {}", input_file.display());
        println!("🔒 Output File: {}", output_path.display());
        println!("🔑 Authority Level: {:?}", authority_level);
        
        // This is a placeholder - in a real implementation, we would:
        // 1. Load the authority chain from the keys directory
        // 2. Find the appropriate authority key for the level
        // 3. Use AuthorityAgeEncryption to perform the encryption
        
        println!("⚠️  Authority-based encryption requires pre-generated authority chain");
        println!("💡 Run 'cli_auth generate' first to create authority keys");
        println!("🔧 Full implementation pending authority chain loading from disk");
        
        Ok(())
    }
    
    fn handle_decrypt(&self, input_file: PathBuf, output_file: PathBuf, authority_level: AuthorityLevelArg, _key_fingerprint: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔓 Authority-Based File Decryption");
        println!("==================================");
        
        println!("🔒 Input File: {}", input_file.display());
        println!("📄 Output File: {}", output_file.display()); 
        println!("🔑 Authority Level: {:?}", authority_level);
        
        println!("⚠️  Authority-based decryption requires loaded authority chain");
        println!("💡 Implementation pending authority chain loading from disk");
        
        Ok(())
    }
    
    fn handle_ignition_create(&self, authority_level: AuthorityLevelArg, passphrase: String, name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Creating Ignition Key");
        println!("========================");
        
        println!("🔑 Authority Level: {:?}", authority_level);
        println!("🏷️  Key Name: {}", name.as_deref().unwrap_or("unnamed"));
        println!("🔐 Passphrase Protection: Enabled");
        
        println!("⚠️  Ignition key creation requires pre-generated authority key");
        println!("💡 Implementation pending authority key loading and IgnitionKey integration");
        
        Ok(())
    }
    
    fn handle_ignition_encrypt(&self, input_file: PathBuf, key_file: PathBuf, _passphrase: String, output_file: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Ignition Key Encryption");
        println!("==========================");
        
        let output_path = output_file.unwrap_or_else(|| {
            input_file.with_extension(format!("{}.age", 
                input_file.extension().and_then(|s| s.to_str()).unwrap_or("txt")
            ))
        });
        
        println!("📄 Input File: {}", input_file.display());
        println!("🚀 Ignition Key: {}", key_file.display());
        println!("🔒 Output File: {}", output_path.display());
        
        println!("⚠️  Ignition key encryption requires passphrase unlocking");
        println!("💡 Implementation pending IgnitionKey workflow integration");
        
        Ok(())
    }
    
    fn handle_validate(&self, test_all: bool, test_pair: Option<String>, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Authority Chain Validation");
        println!("=============================");
        
        if test_all {
            println!("🔗 Testing all authority relationships:");
            let relationships = [
                ("Skull", "Master"),
                ("Master", "Repo"),
                ("Repo", "Ignition"),
                ("Ignition", "Distro"),
            ];
            
            for (parent, child) in &relationships {
                println!("   {} → {} : ⚠️ Requires loaded authority chain", parent, child);
            }
        }
        
        if let Some(pair) = test_pair {
            if let Some((parent, child)) = pair.split_once(':') {
                println!("🔗 Testing specific relationship: {} → {}", parent, child);
                println!("   ⚠️ Requires loaded authority chain");
            } else {
                println!("❌ Invalid pair format. Use 'parent:child' format");
            }
        }
        
        if detailed {
            println!("📊 Detailed validation output enabled");
        }
        
        println!("💡 Full validation requires pre-generated authority chain");
        
        Ok(())
    }
    
    fn handle_status(&self, show_chain: bool, show_keys: bool, show_authorities: bool, name: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Authority Chain Status");
        println!("=========================");
        
        println!("📁 Keys Directory: {}", self.keys_dir.display());
        println!("🎛️  Output Format: {:?}", self.format);
        
        if show_chain {
            println!("\n🔗 Authority Chain Structure:");
            println!("   Skull → Master → Repo → Ignition → Distro");
        }
        
        if show_keys {
            println!("\n🔑 Authority Keys:");
            println!("   🏷️  Chain Name: {}", name);
            // Check for key files using generate pattern: {name}-{type}.key
            let key_types = ["skull", "master", "repo", "ignition", "distro"];
            for key_type in &key_types {
                let key_file = self.keys_dir.join(format!("{}-{}.key", name, key_type));
                if key_file.exists() {
                    println!("   ✅ {} Authority Key: {}", key_type.to_uppercase(), key_file.display());
                } else {
                    println!("   ❌ {} Authority Key: Not found", key_type.to_uppercase());
                }
            }
        }
        
        if show_authorities {
            println!("\n🔐 Authority Relationships:");
            println!("   ⚠️ Authority validation requires loaded chain");
        }
        
        Ok(())
    }
    
    fn handle_test(&self, full_workflow: bool, test_level: Option<AuthorityLevelArg>, benchmark: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Authority Chain Testing");
        println!("==========================");
        
        if full_workflow {
            println!("🔄 Running full end-to-end workflow test");
            println!("   1. Generate authority chain");
            println!("   2. Create ignition keys");
            println!("   3. Test authority-based encryption");
            println!("   4. Test authority-based decryption");
            println!("   5. Validate authority relationships");
            println!("   ⚠️ Full implementation pending");
        }
        
        if let Some(level) = test_level {
            println!("🔑 Testing specific authority level: {:?}", level);
            println!("   ⚠️ Level-specific testing pending");
        }
        
        if benchmark {
            println!("⚡ Running performance benchmarks");
            println!("   ⚠️ Benchmark implementation pending");
        }
        
        Ok(())
    }
    
    fn handle_demo(&self, scenario: DemoScenario, cleanup: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎭 Authority Chain Demonstration");
        println!("================================");
        
        match scenario {
            DemoScenario::Basic => {
                println!("📝 Running Basic Demo:");
                println!("   1. Simple authority key generation");
                println!("   2. Basic encryption/decryption");
            }
            DemoScenario::FullChain => {
                println!("🔗 Running Full Chain Demo:");
                println!("   1. Complete X->M->R->I->D generation");
                println!("   2. Authority relationship validation");
                println!("   3. Multi-level encryption tests");
            }
            DemoScenario::Ignition => {
                println!("🚀 Running Ignition Demo:");
                println!("   1. Ignition key creation with passphrase");
                println!("   2. Ignition-based encryption workflow");
            }
            DemoScenario::Validation => {
                println!("🔍 Running Validation Demo:");
                println!("   1. Authority relationship testing");
                println!("   2. Chain integrity verification");
            }
            DemoScenario::Performance => {
                println!("⚡ Running Performance Demo:");
                println!("   1. Key generation benchmarks");
                println!("   2. Encryption/decryption performance");
            }
        }
        
        if cleanup {
            println!("🧹 Demo cleanup enabled");
        }
        
        println!("⚠️ Full demo implementations are pending");
        
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    
    let mut app = match CliAuth::new(
        cli.verbose,
        cli.keys_dir,
        cli.format.into(),
    ) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("❌ Failed to initialize cli_auth: {}", e);
            process::exit(1);
        }
    };
    
    if let Err(e) = app.run(cli.command) {
        eprintln!("❌ Command failed: {}", e);
        process::exit(1);
    }
}
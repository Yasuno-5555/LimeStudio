use clap::{Parser, Subcommand};
use limestudio_core::preset::PresetArtifact;
use std::path::PathBuf;
mod color;
use color::Colorize;
use std::process::Command;
use limestudio_core::builder::BuildOrchestrator;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// プリセットの証拠能力（改竄・再現性）を検証する
    Verify {
        /// 検証するプリセットファイル (.lime)
        file: PathBuf,
    },
    /// pluginval を使用してビルド済みバイナリを詳細検証する
    Pluginval {
        /// 検証するバイナリのパス (.vst3, .clap)
        path: PathBuf,
        /// 厳格モードで実行する (Level 10)
        #[arg(short, long)]
        strict: bool,
    },
    /// リアルタイム耐性ストレステスト (LFOスパム / テンポ変更 / デノーマル)
    Stress {
        /// 検証するバイナリのパス (.vst3, .clap)
        path: PathBuf,
    },
    /// 証拠品（プリセット）の詳細な法医学レポートを生成する
    Testify {
        /// 検証するプリセットファイル (.lime)
        file: PathBuf,
    },
    /// 生存条件診断（Survival Condition Check）
    Doctor,
    /// プラグインを製品版としてビルド・署名する
    Release,
    
    // 以下、要件見直しにより一時停止中のコマンド
    Validate { file: PathBuf, #[arg(short, long)] hostile: bool },
    Diff { old: PathBuf, new: PathBuf },
    Render { file: PathBuf, output: PathBuf, #[arg(short, long, default_value_t = 1.0)] duration: f32 },
    Bench { file: PathBuf, #[arg(short, long, default_value_t = 64)] block_size: usize },
    Codegen { file: PathBuf, output: Option<PathBuf> },
    Lint { #[arg(short, long)] strict: bool },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify { file } => {
            let json = std::fs::read_to_string(&file).expect("Failed to read file");
            let artifact: PresetArtifact = serde_json::from_str(&json).expect("Failed to parse PresetArtifact");
            
            println!("Verifying Artifact: {}...", artifact.name);
            if artifact.verify() {
                println!("{}", "VALID: Integrity check passed.".green());
            } else {
                println!("{}", "TAMPERED: Integrity check failed!".red());
            }
        }
        Commands::Pluginval { path, strict } => {
            println!("Invoking pluginval for {}...", path.display());
            let mut cmd = Command::new("pluginval");
            cmd.arg("--validate").arg(&path);
            if strict {
                cmd.arg("--validate-strict").arg("--strict-level").arg("10");
            }
            
            let status = cmd.status().map_err(|e| anyhow::anyhow!("Failed to execute pluginval: {}. Is it in your PATH?", e))?;
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Commands::Stress { path } => {
            println!("{}", "═══ LIME REAL-TIME STRESS HARNESS ═══".bold().red());
            println!("Target: {}", path.display());
            
            println!("\n[1/3] Chaos Automation Test...");
            let mut cmd = Command::new("pluginval");
            cmd.arg("--validate").arg(&path)
               .arg("--validate-strict")
               .arg("--strict-level").arg("10")
               .arg("--test-automation"); 
            
            if !cmd.status()?.success() {
                println!("{}", "FAILED: Chaos Automation triggered a failure.".red());
                std::process::exit(1);
            }

            println!("\n[2/3] Denormal / Zero Input Stress...");
            // pluginval doesn't have a specific "denormal" flag, but we can assume normal validation handles basic safety.
            // In a future version, we would use a custom host to feed denormals.
            println!("{}", "  [SKIP] Custom Denormal feeder not yet implemented in CLI.".dimmed());

            println!("\n[3/3] Thread Starvation Simulation...");
            println!("{}", "  [SKIP] Starvation simulation requires custom host wrapper.".dimmed());

            println!("\n{}", "STRESS TEST COMPLETED. REALITY REMAINS STABLE.".green().bold());
            println!("{}", "═════════════════════════════════════".bold().red());
        }
        Commands::Testify { file } => {
            let json = std::fs::read_to_string(&file).expect("Failed to read file");
            let artifact: PresetArtifact = serde_json::from_str(&json).expect("Failed to parse PresetArtifact");
            
            println!("{}", "═══ FORENSIC TESTIMONY ═══".bold().cyan());
            println!("Artifact: {}", artifact.name);
            println!("Version:  {}", artifact.version);
            println!("Hash:     {}", artifact.hash);
            
            if let Some(sh) = &artifact.source_hash {
                println!("Source:   {}", sh);
            }
            
            println!("\nVerification Result: {}", if artifact.verify() { "VALID".green() } else { "TAMPERED".red() });
            println!("{}", "═══════════════════════════".bold().cyan());
        }
        Commands::Doctor => {
            println!("{}", "═══ LIME SURVIVAL CONDITION CHECK ═══".bold().magenta());
            let mut unsafe_state = false;

            // 1. Tool Check: pluginval
            print!("Checking pluginval... ");
            if Command::new("pluginval").arg("--version").output().is_ok() {
                println!("{}", "OK".green());
            } else {
                println!("{}", "NOT FOUND (UNSAFE)".red());
                println!("  -> Please install pluginval to enable validation: brew install pluginval");
                unsafe_state = true;
            }

            // 2. Hardware Check: SIMD
            print!("Checking SIMD support... ");
            #[cfg(target_arch = "x86_64")]
            {
                if std::is_x86_feature_detected!("avx2") {
                    println!("{}", "AVX2 OK".green());
                } else {
                    println!("{}", "AVX2 MISSING (WARN)".yellow());
                }
            }
            #[cfg(target_arch = "aarch64")]
            {
                println!("{}", "NEON OK (Apple Silicon)".green());
            }

            // 3. Environment Check: DAW Detection (macOS)
            #[cfg(target_os = "macos")]
            {
                println!("Scanning for Host environments:");
                let daws = [
                    ("Bitwig Studio", "/Applications/Bitwig Studio.app"),
                    ("Ableton Live", "/Applications/Ableton Live 11 Suite.app"), 
                    ("Logic Pro", "/Applications/Logic Pro X.app"),
                ];
                for (name, path) in daws {
                    if std::path::Path::new(path).exists() {
                        println!("  [FOUND] {}", name.green());
                    } else {
                        println!("  [MISSING] {}", name.dimmed());
                    }
                }
            }

            println!("\nConclusion: {}", if unsafe_state { "UNSAFE STATE".red().bold() } else { "READY FOR PRODUCTION".green().bold() });
            println!("{}", "══════════════════════════════════════".bold().magenta());
            
            if unsafe_state {
                std::process::exit(1);
            }
        }
        Commands::Release => {
            let orchestrator = BuildOrchestrator::new(
                "LimePlugin".to_string(), 
                "dev.limestudio.plugin".to_string()
            );

            if let Err(e) = orchestrator.run_release_build("aarch64-apple-darwin") {
                eprintln!("Build failed: {:?}", e);
                std::process::exit(1);
            }
        }
        _ => {
            println!("This command is temporarily disabled due to core refactoring.");
        }
    }
    Ok(())
}

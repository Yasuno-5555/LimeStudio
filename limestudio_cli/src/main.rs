use clap::{Parser, Subcommand};
use limestudio_core::graph::AudioGraph;
use limestudio_core::preset::{Preset, diagnose_preset};
use limestudio_core::hostile::validate_hostile;
use limestudio_core::validate::validate_graph;
use limestudio_core::compile::compile_graph;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// プリセット/グラフファイルを詳細検査する
    Inspect {
        /// ファイルパス (.json または .lime)
        file: PathBuf,
    },
    /// Hostile Validation を実行する
    Validate {
        /// ファイルパス (.json または .lime)
        file: PathBuf,
        /// より残酷な検証を行う
        #[arg(short, long)]
        hostile: bool,
    },
    /// プリセットの健全性をチェックする (Preset Doctor)
    Doctor {
        /// プリセットファイルのパス (.lime)
        file: PathBuf,
    },
    /// 2つのプリセットの差分を表示する
    Diff {
        /// 以前のプリセット
        old: PathBuf,
        /// 新しいプリセット
        new: PathBuf,
    },
    /// 指定された時間 (秒) だけオーディオをレンダリングして出力する
    Render {
        /// プリセットファイル (.lime)
        file: PathBuf,
        /// 出力ファイル名 (.wav)
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,
        /// レンダリング時間 (秒)
        #[arg(short, long, default_value_t = 1.0)]
        duration: f32,
    },
    /// パフォーマンス計測 (ベンチマーク)
    Bench {
        /// プリセットファイル (.lime)
        file: PathBuf,
        /// ブロックサイズ
        #[arg(short, long, default_value_t = 512)]
        block_size: usize,
    },
    /// Rust コードをエクスポートする
    Codegen {
        /// プリセットファイル (.lime)
        file: PathBuf,
        /// 出力ファイル名
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { file } => {
            let (graph, _preset) = load_file(&file);
            graph.debug_dump();
            
            match validate_graph(&graph) {
                Ok(order) => {
                    let program = compile_graph(&graph, &order).program;
                    program.pretty_print();
                }
                Err(e) => {
                    println!("Basic Validation failed: {:?}", e);
                }
            }
        }
        Commands::Validate { file, hostile } => {
            let (graph, _preset) = load_file(&file);
            
            if hostile {
                println!("Running Hostile Validation...");
                match validate_graph(&graph) {
                    Ok(order) => {
                        let program = compile_graph(&graph, &order).program;
                        let report = validate_hostile(&graph, &program);
                        report.print_report();
                        if report.has_errors() {
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        println!("Structural Validation failed: {:?}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match validate_graph(&graph) {
                    Ok(_) => println!("Structural Validation successful."),
                    Err(e) => {
                        println!("Validation failed: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Doctor { file } => {
            let json = std::fs::read_to_string(&file).expect("Failed to read file");
            let preset = Preset::from_json(&json).expect("Failed to parse preset");
            let report = diagnose_preset(&preset);
            
            println!("═══ Preset Doctor Report: {} ═══", file.display());
            for issue in &report.issues {
                println!("  [{:?}] {}: {}", issue.severity, issue.code, issue.message);
            }
            if report.has_errors() {
                std::process::exit(1);
            }
        }
        Commands::Diff { old, new } => {
            let json_old = std::fs::read_to_string(old).expect("Failed to read old file");
            let json_new = std::fs::read_to_string(new).expect("Failed to read new file");
            
            let preset_old = Preset::from_json(&json_old).expect("Failed to parse old preset");
            let preset_new = Preset::from_json(&json_new).expect("Failed to parse new preset");
            
            let diff = limestudio_core::diff::diff_presets(&preset_old, &preset_new);
            diff.print_summary();
        }
        Commands::Render { file, output, duration } => {
            let (graph, _) = load_file(&file);
            println!("Rendering {} for {:.1}s...", file.display(), duration);
            
            let order = validate_graph(&graph).expect("Validation failed");
            let program = compile_graph(&graph, &order).program;
            
            let samples = (duration * 44100.0) as usize; // Assume 44.1k for CLI render
            let dummy_input = vec![0.0f32; samples];
            let output_samples = limestudio_core::golden::render_program(program, 44100, samples, &dummy_input);
            
            // Note: In a real implementation, we'd use a WAV encoder crate here.
            // For now, we confirm deterministic completion.
            println!("Render completed successfully. (Simulated output: {})", output.display());
            println!("Output Samples: {}", output_samples.len());
        }
        Commands::Bench { file, block_size } => {
            let (graph, _) = load_file(&file);
            println!("Benchmarking {} (Block size: {})...", file.display(), block_size);
            
            let order = validate_graph(&graph).expect("Validation failed");
            let program = compile_graph(&graph, &order).program;
            let mut engine = limestudio_core::engine::DspEngine::new_from_program(program);
            
            let mut inputs = vec![vec![0.0f32; block_size]; 2];
            let mut outputs = vec![vec![0.0f32; block_size]; 2];
            
            let start = std::time::Instant::now();
            let iterations = 1000;
            
            // Slice conversion boilerplate
            let input_slices: Vec<&[f32]> = inputs.iter().map(|v| v.as_slice()).collect();
            let mut output_slices: Vec<&mut [f32]> = outputs.iter_mut().map(|v| v.as_mut_slice()).collect();
            
            for _ in 0..iterations {
                engine.process_block(&input_slices, &mut output_slices);
            }
            let elapsed = start.elapsed();
            let per_block = elapsed / iterations as u32;
            
            println!("═══ Benchmark Results ═══");
            println!("  Total time (1000 blocks): {:?}", elapsed);
            println!("  Time per block:          {:?}", per_block);
            println!("  Real-time safety margin: {:.2}%", (per_block.as_secs_f32() / (block_size as f32 / 44100.0)) * 100.0);
        }
        Commands::Codegen { file, output } => {
            let (graph, _) = load_file(&file);
            println!("Generating Rust code for {}...", file.display());
            
            let order = validate_graph(&graph).expect("Validation failed");
            let program = compile_graph(&graph, &order).program;
            let code = limestudio_core::codegen::ir_to_readable_rust(&program);
            
            if let Some(out_path) = output {
                std::fs::write(&out_path, &code).expect("Failed to write output file");
                println!("Code exported to: {}", out_path.display());
            } else {
                println!("═══ Generated Rust Code ═══");
                println!("{}", code);
                println!("═══════════════════════════");
            }
        }
    }
}

fn load_file(path: &PathBuf) -> (AudioGraph, Option<Preset>) {
    let json = std::fs::read_to_string(path).expect("Failed to read file");
    
    // Try loading as Preset first
    if let Ok(preset) = Preset::from_json(&json) {
        (preset.graph_snapshot.clone(), Some(preset))
    } else {
        // Fallback to raw AudioGraph
        let graph = AudioGraph::from_json(&json).expect("Failed to parse graph JSON");
        (graph, None)
    }
}

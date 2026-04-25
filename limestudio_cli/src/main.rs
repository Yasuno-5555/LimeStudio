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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { file } => {
            let (graph, _preset) = load_file(&file);
            graph.debug_dump();
            
            match validate_graph(&graph) {
                Ok(order) => {
                    let program = compile_graph(&graph, &order);
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
                        let program = compile_graph(&graph, &order);
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

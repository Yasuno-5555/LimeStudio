use clap::{Parser, Subcommand};
use limestudio_core::graph::AudioGraph;
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
    /// グラフファイルを検査し、構造を表示する
    Inspect {
        /// グラフファイルのパス (.json)
        file: PathBuf,
    },
    /// グラフファイルを検証する (サイクル検知など)
    Validate {
        /// グラフファイルのパス (.json)
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { file } => {
            let json = std::fs::read_to_string(file).expect("Failed to read file");
            let graph = AudioGraph::from_json(&json).expect("Failed to parse graph JSON");
            
            graph.debug_dump();
            
            match validate_graph(&graph) {
                Ok(order) => {
                    let program = compile_graph(&graph, &order);
                    program.pretty_print();
                }
                Err(e) => {
                    println!("Validation failed: {:?}", e);
                }
            }
        }
        Commands::Validate { file } => {
            let json = std::fs::read_to_string(file).expect("Failed to read file");
            let graph = AudioGraph::from_json(&json).expect("Failed to parse graph JSON");
            
            match validate_graph(&graph) {
                Ok(order) => {
                    println!("Validation successful.");
                    println!("Execution Order: {:?}", order);
                }
                Err(e) => {
                    println!("Validation failed: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

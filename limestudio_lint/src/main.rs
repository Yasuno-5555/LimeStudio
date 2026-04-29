use clap::Parser;
use colored::*;
use std::fs;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum Cargo {
    LimeLint(LintArgs),
}

#[derive(clap::Args, Debug)]
struct LintArgs {
    #[arg(short, long)]
    strict: bool,
    #[arg(long)]
    dramatic: bool,
}

#[derive(serde::Deserialize)]
struct Config {
    _realtime: RealtimeConfig,
}

#[derive(serde::Deserialize)]
struct RealtimeConfig {
    _forbidden_calls: std::collections::HashMap<String, String>,
}

fn main() -> anyhow::Result<()> {
    let Cargo::LimeLint(args) = Cargo::parse();

    let config_str = fs::read_to_string("lime-lint.toml").unwrap_or_default();
    let _config: Config = toml::from_str(&config_str).unwrap();

    println!("{}", "LimeLint v0.1.0 — Law Edition".bold().green());
    if args.dramatic {
        println!(
            "{}",
            "The Judge has arrived. Prepare for clinical examination.".red()
        );
    }

    let mut violations = 0;

    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "rs") {
            let content = fs::read_to_string(entry.path())?;
            if content.contains("plugin!") {
                violations += lint_file(entry.path().to_str().unwrap(), &content, args.strict)?;
            }
        }
    }

    if violations == 0 {
        println!(
            "\n{}",
            "Result: TRUSTWORTHY. All visual laws observed."
                .bold()
                .green()
        );
    } else {
        println!(
            "\n{}",
            format!(
                "Result: FAILED. {} violations found. Fix your UI or face the consequences.",
                violations
            )
            .bold()
            .red()
        );
        std::process::exit(1);
    }

    Ok(())
}

fn lint_file(path: &str, content: &str, strict: bool) -> anyhow::Result<usize> {
    let mut count = 0;

    // Simple regex/string checks for Phase 1
    // We will upgrade to syn visitor in the next step

    if content.contains(".shadow(true)") {
        report_violation(
            path,
            "Forbidden: .shadow(true) is not allowed in Lime HIG v3.0.",
            "Matte Rule 2.1",
            strict,
        );
        count += 1;
    }

    if content.contains(".glow(") {
        report_violation(
            path,
            "Forbidden: .glow(...) is a crime against clarity.",
            "Matte Rule 2.1",
            strict,
        );
        count += 1;
    }

    if content.contains("Color::rgb(") {
        report_violation(
            path,
            "Warning: Raw RGB detected. Use Oklab constants for perceptual uniformity.",
            "Color Rule 2.3",
            strict,
        );
        if strict {
            count += 1;
        }
    }

    Ok(count)
}

fn report_violation(file: &str, msg: &str, rule: &str, dramatic: bool) {
    println!(
        "\n{}",
        format!("[{}] Violation found in {}", rule, file)
            .bold()
            .red()
    );
    if dramatic {
        // Dramatic mode: Merciless
        match rule {
            "Matte Rule 2.1" => println!(
                "  {}",
                "Your UI has chosen chaos. Theatrical lighting is a moral failure.".red()
            ),
            _ => println!("  {}", format!("CRIME: {}", msg).red()),
        }
    } else {
        // Clinical mode: Accurate
        println!("  {}", msg.yellow());
    }
}

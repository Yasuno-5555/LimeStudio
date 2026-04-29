use once_cell::sync::Lazy;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 死体（クラッシュレポート）をメモリ上に保存するためのバッファ
static LAST_CRASH_REPORT: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub trait CrashReporter: Send + Sync {
    fn dump_state(&self, writer: &mut dyn Write) -> std::io::Result<()>;
}

/// 最後に発生したクラッシュレポートをメモリから取得する
pub fn get_last_crash_report() -> Option<String> {
    LAST_CRASH_REPORT.lock().unwrap().clone()
}

fn get_log_paths(plugin_name: &str, timestamp: &str) -> Vec<PathBuf> {
    let filename = format!(
        "LIME_CRASH_{}_{}.txt",
        plugin_name.replace(" ", "_"),
        timestamp
    );
    let mut paths = Vec::new();

    // 1. OS標準ログディレクトリ (macOS)
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push("Library/Logs/LimeStudio");
        paths.push(p.join(&filename));
    }

    // 2. ユーザードキュメント
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push("Documents/LimeStudio/Logs");
        paths.push(p.join(&filename));
    }

    // 3. カレントディレクトリ (Last resort)
    paths.push(PathBuf::from(&filename));

    paths
}

pub fn install_handler(reporter: Arc<dyn CrashReporter>, plugin_name: &'static str) {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let timestamp = limestudio_core::time::Timestamp::now().to_string();

        // メモリバッファへの書き込み準備
        let mut mem_buffer = Vec::new();

        let write_to = |w: &mut dyn Write| -> std::io::Result<()> {
            writeln!(w, "═══ LIME FORENSIC CRASH REPORT ═══")?;
            writeln!(w, "Plugin: {}", plugin_name)?;
            writeln!(w, "Time:   {}", limestudio_core::time::Timestamp::now())?;
            writeln!(w, "Cause:  {:?}", panic_info)?;

            writeln!(w, "\n--- Parameter State ---")?;
            reporter.dump_state(w)?;

            writeln!(w, "\n═══ END OF REPORT ═══")?;
            Ok(())
        };

        // 1. メモリに保存 (常に実行)
        let _ = write_to(&mut mem_buffer);
        if let Ok(s) = String::from_utf8(mem_buffer) {
            *LAST_CRASH_REPORT.lock().unwrap() = Some(s);
        }

        // 2. ファイルに保存 (優先順位に従って試行)
        for path in get_log_paths(plugin_name, &timestamp) {
            if let Some(parent) = path.parent() {
                let _ = create_dir_all(parent);
            }
            if let Ok(mut file) = File::create(&path) {
                if let Some(report) = &*LAST_CRASH_REPORT.lock().unwrap() {
                    let bytes: &[u8] = report.as_bytes();
                    let _ = file.write_all(bytes);
                    break; // 成功したら終了
                }
            }
        }

        original_hook(panic_info);
    }));
}

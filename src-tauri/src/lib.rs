use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub line_number: u32,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub path: String,
    pub engine: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub max_depth: Option<u32>,
    pub file_types: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

async fn search_with_ripgrep(app: AppHandle, params: SearchParams) -> Result<Vec<SearchResult>, String> {
    let command = app.shell().command("rg")
        .arg("--json")
        .arg("--line-number");

    let command = if !params.case_sensitive {
        command.args(["-i"])
    } else {
        command
    };

    let command = if params.whole_word {
        command.args(["-w"])
    } else {
        command
    };

    let command = if !params.use_regex {
        command.args(["--fixed-strings"])
    } else {
        command
    };

    let command = if let Some(depth) = params.max_depth {
        let depth_str = depth.to_string();
        command.args(["--max-depth", &depth_str])
    } else {
        command
    };

    // Handle file types
    let command = params.file_types.iter().fold(command, |cmd, file_type| {
        cmd.args(["-g", file_type])
    });

    // Handle exclude patterns
    let command = params.exclude_patterns.iter().fold(command, |cmd, pattern| {
        let glob_pattern = format!("!{}", pattern);
        cmd.args(["--glob", &glob_pattern])
    });

    let command = command.arg(&params.pattern).arg(&params.path);

    let output = command.output().await.map_err(|e| e.to_string())?;

    if !output.status.success() && !output.status.code().map_or(false, |c| c == 1) {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ripgrep search failed: {}", error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if let Ok(output) = serde_json::from_str::<RipgrepOutput>(line) {
            if let RipgrepOutput::Match { data } = output {
                results.push(SearchResult {
                    path: data.path.text,
                    line_number: data.line_number as u32,
                    content: data.lines.text,
                });
            }
        }
    }

    Ok(results)
}

async fn search_with_grep(app: AppHandle, params: SearchParams) -> Result<Vec<SearchResult>, String> {
    let command = app.shell().command("grep");

    let mut args = vec!["--line-number", "--with-filename"];

    if !params.case_sensitive {
        args.push("--ignore-case");
    }

    if params.whole_word {
        args.push("--word-regexp");
    }

    if params.use_regex {
        args.push("--extended-regexp");
    } else {
        args.push("--fixed-strings");
    }

    args.push(&params.pattern);
    args.push(&params.path);

    let output = command
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() && !output.status.code().map_or(false, |c| c == 1) {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("grep search failed: {}", error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if let Some((path, rest)) = line.split_once(':') {
            if let Some((line_number_str, content)) = rest.split_once(':') {
                if let Ok(line_number) = line_number_str.parse::<u32>() {
                    results.push(SearchResult {
                        path: path.to_string(),
                        line_number,
                        content: content.to_string(),
                    });
                }
            }
        }
    }

    Ok(results)
}


#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum RipgrepOutput {
    #[serde(rename = "match")]
    Match {
        data: MatchData,
    },
    #[serde(rename = "summary")]
    Summary { data: SummaryData },
}

#[derive(Debug, Deserialize)]
struct MatchData {
    path: PathData,
    lines: TextData,
    line_number: u64,
}

#[derive(Debug, Deserialize)]
struct PathData {
    text: String,
}

#[derive(Debug, Deserialize)]
struct TextData {
    text: String,
}

#[derive(Debug, Deserialize)]
struct SummaryData {
    elapsed_total: Elapsed,
    stats: SearchStats,
}

#[derive(Debug, Deserialize)]
struct SearchStats {
    elapsed: Elapsed,
    searches: u64,
    searches_with_match: u64,
    bytes_searched: u64,
    bytes_printed: u64,
    matched_lines: u64,
    matches: u64,
}

#[derive(Debug, Deserialize)]
struct Elapsed {
    secs: u64,
    nanos: u64,
    human: String,
}

pub async fn perform_search(app: AppHandle, params: SearchParams) -> Result<Vec<SearchResult>, String> {
    match params.engine.as_str() {
        "ripgrep" => search_with_ripgrep(app, params).await,
        "grep" => search_with_grep(app, params).await,
        _ => Err("Unsupported search engine".to_string()),
    }
}

#[tauri::command]
async fn search(app: AppHandle, params: SearchParams) -> Result<Vec<SearchResult>, String> {
    perform_search(app, params).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = fix_path_env::fix();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

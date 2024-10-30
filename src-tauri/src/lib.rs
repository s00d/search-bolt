use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    path: String,
    line_number: u32,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    path: String,
    engine: String,
    pattern: String,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    max_depth: Option<u32>,
    file_types: Vec<String>,
    exclude_patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum RipgrepOutput {
    #[serde(rename = "match")]
    Match {
        data: MatchData,
    },
    // #[serde(rename = "begin")]
    // Begin { data: BeginData },
    // #[serde(rename = "end")]
    // End { data: EndData },
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
struct BeginData {
    path: PathData,
}

#[derive(Debug, Deserialize)]
struct EndData {
    path: PathData,
    stats: SearchStats,
}

#[derive(Debug, Deserialize)]
struct SummaryData {
    elapsed_total: Elapsed,
    stats: SearchStats,
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


async fn search_with_ripgrep(params: SearchParams) -> Result<Vec<SearchResult>, String> {
    let mut cmd = Command::new("rg");

    cmd.arg("--json")
        .arg("--line-number");

    if !params.case_sensitive {
        cmd.arg("-i");
    }

    if params.whole_word {
        cmd.arg("-w");
    }

    if !params.use_regex {
        cmd.arg("--fixed-strings");
    }

    if let Some(depth) = params.max_depth {
        cmd.arg("--max-depth").arg(depth.to_string());
    }

    for file_type in params.file_types {
        cmd.arg("-g").arg(file_type);
    }

    for pattern in params.exclude_patterns {
        cmd.arg("--glob").arg(format!("!{}", pattern));
    }

    cmd.arg(&params.pattern)
        .arg(&params.path);

    let output = cmd.output().map_err(|e| e.to_string())?;

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

async fn search_with_grep(params: SearchParams) -> Result<Vec<SearchResult>, String> {
    let mut cmd = Command::new("grep");

    cmd.arg("--line-number")  // Show line numbers
        .arg("--with-filename"); // Show filenames

    if !params.case_sensitive {
        cmd.arg("--ignore-case");
    }

    if params.whole_word {
        cmd.arg("--word-regexp");
    }

    if params.use_regex {
        cmd.arg("--extended-regexp");
    } else {
        cmd.arg("--fixed-strings");
    }

    // Add pattern and path
    cmd.arg(&params.pattern)
        .arg(&params.path);

    let output = cmd.output().map_err(|e| e.to_string())?;

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

pub async fn perform_search(params: SearchParams) -> Result<Vec<SearchResult>, String> {
    match params.engine.as_str() {
        "ripgrep" => search_with_ripgrep(params).await,
        "grep" => search_with_grep(params).await,
        _ => Err("Unsupported search engine".to_string()),
    }
}


#[tauri::command]
async fn search(params: SearchParams) -> Result<Vec<SearchResult>, String> {
    perform_search(params).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use grep_regex::RegexMatcherBuilder;
use grep_matcher::Matcher;
use grep_printer::StandardBuilder;
use grep_searcher::{sinks::Lossy, SearcherBuilder};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use log::{error, info, warn};
use opensesame::{Editor, EditorKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QuickWindowSizes {
    pub width: u32,
    pub idle_height: u32,
    pub searching_height: u32,
    pub no_results_height: u32,
    pub error_height: u32,
    pub results_height: u32,
}

impl Default for QuickWindowSizes {
    fn default() -> Self {
        Self {
            width: 980,
            idle_height: 760,
            searching_height: 820,
            no_results_height: 905,
            error_height: 995,
            results_height: 1145,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchDefaults {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub literal: bool,
    pub multiline: bool,
    pub before_context: usize,
    pub after_context: usize,
    pub engine: SearchEngine,
    pub binary_policy: BinaryPolicy,
    pub max_depth: Option<u32>,
    pub file_types: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub page_size: usize,
    pub max_results: usize,
    pub timeout_seconds: u64,
}

impl Default for SearchDefaults {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            literal: false,
            multiline: false,
            before_context: 0,
            after_context: 0,
            engine: SearchEngine::RustRegex,
            binary_policy: BinaryPolicy::Lossy,
            max_depth: None,
            file_types: Vec::new(),
            exclude_patterns: Vec::new(),
            page_size: 50,
            max_results: 100,
            timeout_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistorySettings {
    pub limit: usize,
}

impl Default for HistorySettings {
    fn default() -> Self {
        Self { limit: 20 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TraySettings {
    pub menu_on_left_click: bool,
}

impl Default for TraySettings {
    fn default() -> Self {
        Self {
            menu_on_left_click: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorSettings {
    pub search_as_you_type: bool,
    pub search_debounce_ms: u64,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            search_as_you_type: true,
            search_debounce_ms: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchProfile {
    pub id: String,
    pub name: String,
    pub file_types: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for SearchProfile {
    fn default() -> Self {
        Self {
            id: "everything".to_string(),
            name: "Everything".to_string(),
            file_types: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedSearchPreset {
    pub id: String,
    pub name: String,
    pub path: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub literal: bool,
    pub multiline: bool,
    pub before_context: usize,
    pub after_context: usize,
    pub engine: SearchEngine,
    pub binary_policy: BinaryPolicy,
    pub max_depth: Option<u32>,
    pub file_types: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub page_size: usize,
    pub max_results: usize,
    pub timeout_seconds: u64,
}

impl Default for SavedSearchPreset {
    fn default() -> Self {
        let defaults = SearchDefaults::default();
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            path: String::new(),
            pattern: String::new(),
            case_sensitive: defaults.case_sensitive,
            whole_word: defaults.whole_word,
            use_regex: defaults.use_regex,
            literal: defaults.literal,
            multiline: defaults.multiline,
            before_context: defaults.before_context,
            after_context: defaults.after_context,
            engine: defaults.engine,
            binary_policy: defaults.binary_policy,
            max_depth: defaults.max_depth,
            file_types: defaults.file_types,
            exclude_patterns: defaults.exclude_patterns,
            page_size: defaults.page_size,
            max_results: defaults.max_results,
            timeout_seconds: defaults.timeout_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditorMode {
    System,
    Preset,
    CustomTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditorPreset {
    VsCode,
    VsCodeInsiders,
    VSCodium,
    Cursor,
    Windsurf,
    Zed,
    Sublime,
    Vim,
    NeoVim,
    Emacs,
    Helix,
    Nano,
    TextMate,
    NotepadPlusPlus,
    Xcode,
    JetBrains,
    JetBrainsIntelliJ,
    JetBrainsWebStorm,
    JetBrainsPhpStorm,
    JetBrainsPyCharm,
    JetBrainsRubyMine,
    JetBrainsGoLand,
    JetBrainsCLion,
    JetBrainsRider,
    JetBrainsDataGrip,
    JetBrainsAndroidStudio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    pub mode: EditorMode,
    pub preset: EditorPreset,
    pub custom_template: String,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            mode: EditorMode::System,
            preset: EditorPreset::VsCode,
            custom_template: "code -g \"{path}:{line}:{column}\"".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub hotkey: String,
    pub autostart: bool,
    pub prefill_strategy: String,
    pub search_defaults: SearchDefaults,
    pub quick_window_sizes: QuickWindowSizes,
    pub history: HistorySettings,
    pub tray: TraySettings,
    pub behavior: BehaviorSettings,
    pub search_profiles: Vec<SearchProfile>,
    pub saved_searches: Vec<SavedSearchPreset>,
    pub editor: EditorSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "CommandOrControl+Shift+F".to_string(),
            autostart: false,
            prefill_strategy: "active_then_clipboard_then_history".to_string(),
            search_defaults: SearchDefaults::default(),
            quick_window_sizes: QuickWindowSizes::default(),
            history: HistorySettings::default(),
            tray: TraySettings::default(),
            behavior: BehaviorSettings::default(),
            search_profiles: default_search_profiles(),
            saved_searches: Vec::new(),
            editor: EditorSettings::default(),
        }
    }
}

fn default_search_profiles() -> Vec<SearchProfile> {
    vec![
        SearchProfile {
            id: "code".to_string(),
            name: "Code".to_string(),
            file_types: Vec::new(),
            exclude_patterns: vec![
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
                "target/**".to_string(),
                "vendor/**".to_string(),
            ],
        },
        SearchProfile {
            id: "frontend".to_string(),
            name: "Frontend".to_string(),
            file_types: vec![
                "*.ts".to_string(),
                "*.tsx".to_string(),
                "*.js".to_string(),
                "*.jsx".to_string(),
                "*.vue".to_string(),
                "*.css".to_string(),
                "*.scss".to_string(),
                "*.html".to_string(),
            ],
            exclude_patterns: vec![
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                "coverage/**".to_string(),
                ".git/**".to_string(),
            ],
        },
        SearchProfile {
            id: "rust".to_string(),
            name: "Rust".to_string(),
            file_types: vec!["*.rs".to_string(), "*.toml".to_string()],
            exclude_patterns: vec!["target/**".to_string(), ".git/**".to_string()],
        },
        SearchProfile::default(),
    ]
}

struct RuntimeState {
    settings: Mutex<AppSettings>,
    current_hotkey: Mutex<String>,
    autostart_menu_item: Mutex<Option<CheckMenuItem<tauri::Wry>>>,
    search_sessions: Mutex<HashMap<String, SearchSession>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub line_number: u32,
    pub content: String,
    #[serde(default)]
    pub before_context: Vec<String>,
    #[serde(default)]
    pub after_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchEngine {
    RustRegex,
    Pcre2,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::RustRegex
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BinaryPolicy {
    Skip,
    Lossy,
}

impl Default for BinaryPolicy {
    fn default() -> Self {
        Self::Lossy
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    pub path: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    #[serde(default)]
    pub literal: bool,
    #[serde(default)]
    pub multiline: bool,
    #[serde(default)]
    pub before_context: usize,
    #[serde(default)]
    pub after_context: usize,
    #[serde(default)]
    pub engine: SearchEngine,
    #[serde(default)]
    pub binary_policy: BinaryPolicy,
    pub max_depth: Option<u32>,
    pub file_types: Vec<String>,
    pub exclude_patterns: Vec<String>,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

fn default_max_results() -> usize {
    100
}

fn default_timeout_seconds() -> u64 {
    60
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPage {
    pub items: Vec<SearchResult>,
    pub has_more: bool,
    pub next_cursor: Option<usize>,
    pub total: usize,
    pub stats: SearchPageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPageStats {
    pub total_matches: usize,
    pub returned_in_page: usize,
    pub page_size: usize,
    pub elapsed_ms: u128,
    pub files_scanned: usize,
    pub files_skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStartResponse {
    pub session_id: String,
    pub page: SearchPage,
}

#[derive(Debug)]
struct SearchSession {
    items: Vec<SearchResult>,
    page_size: usize,
    elapsed_ms: u128,
    files_scanned: usize,
    files_skipped: usize,
}

#[derive(Debug, Clone)]
struct SearchRunStats {
    elapsed_ms: u128,
    files_scanned: usize,
    files_skipped: usize,
}
const SETTINGS_STORE_PATH: &str = "settings.json";
const SETTINGS_KEY: &str = "appSettings";

fn load_settings(app: &AppHandle) -> AppSettings {
    let Ok(store) = app.store(SETTINGS_STORE_PATH) else {
        return AppSettings::default();
    };
    let Some(value) = store.get(SETTINGS_KEY) else {
        let defaults = AppSettings::default();
        let _ = save_settings(app, &defaults);
        return defaults;
    };
    match serde_json::from_value::<AppSettings>(value) {
        Ok(settings) => settings,
        Err(_) => AppSettings::default(),
    }
}

fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let store = app.store(SETTINGS_STORE_PATH).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, value);
    store.save().map_err(|e| e.to_string())
}

fn apply_hotkey(app: &AppHandle, old_hotkey: &str, new_hotkey: &str) -> Result<(), String> {
    if old_hotkey == new_hotkey {
        return Ok(());
    }

    if !old_hotkey.trim().is_empty() {
        let _ = app.global_shortcut().unregister(old_hotkey);
    }
    app.global_shortcut()
        .register(new_hotkey)
        .map_err(|e| format!("Invalid hotkey '{new_hotkey}': {e}"))?;
    Ok(())
}

fn resolve_editor_kind(preset: &EditorPreset) -> EditorKind {
    match preset {
        EditorPreset::VsCode => EditorKind::VsCode,
        EditorPreset::VsCodeInsiders => EditorKind::VsCodeInsiders,
        EditorPreset::VSCodium => EditorKind::VSCodium,
        EditorPreset::Cursor => EditorKind::Cursor,
        EditorPreset::Windsurf => EditorKind::Windsurf,
        EditorPreset::Zed => EditorKind::Zed,
        EditorPreset::Sublime => EditorKind::Sublime,
        EditorPreset::Vim => EditorKind::Vim,
        EditorPreset::NeoVim => EditorKind::NeoVim,
        EditorPreset::Emacs => EditorKind::Emacs,
        EditorPreset::Helix => EditorKind::Helix,
        EditorPreset::Nano => EditorKind::Nano,
        EditorPreset::TextMate => EditorKind::TextMate,
        EditorPreset::NotepadPlusPlus => EditorKind::NotepadPlusPlus,
        EditorPreset::Xcode => EditorKind::Xcode,
        // Generic JetBrains launcher (usually `idea` binary).
        EditorPreset::JetBrains => EditorKind::IntelliJ,
        EditorPreset::JetBrainsIntelliJ => EditorKind::IntelliJ,
        EditorPreset::JetBrainsWebStorm => EditorKind::WebStorm,
        EditorPreset::JetBrainsPhpStorm => EditorKind::PhpStorm,
        EditorPreset::JetBrainsPyCharm => EditorKind::PyCharm,
        EditorPreset::JetBrainsRubyMine => EditorKind::RubyMine,
        EditorPreset::JetBrainsGoLand => EditorKind::GoLand,
        EditorPreset::JetBrainsCLion => EditorKind::CLion,
        EditorPreset::JetBrainsRider => EditorKind::Rider,
        EditorPreset::JetBrainsDataGrip => EditorKind::DataGrip,
        EditorPreset::JetBrainsAndroidStudio => EditorKind::AndroidStudio,
    }
}

fn open_with_system_default(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(path);
        c
    };
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(path);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", path]);
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err("System editor open is not supported on this platform".to_string());

    cmd.spawn()
        .map_err(|e| format!("Failed to open file with system default app: {e}"))?;
    Ok(())
}

fn open_with_custom_template(
    template: &str,
    path: &str,
    line: u32,
    column: u32,
) -> Result<(), String> {
    if template.trim().is_empty() {
        return Err("Custom editor template is empty".to_string());
    }

    let rendered = template
        .replace("{path}", path)
        .replace("{line}", &line.to_string())
        .replace("{column}", &column.to_string());
    let argv = shlex::split(&rendered)
        .ok_or_else(|| "Failed to parse custom editor template (quote mismatch)".to_string())?;
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| "Custom editor template must include a program".to_string())?;

    Command::new(program)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to spawn custom editor command: {e}"))?;
    Ok(())
}

fn open_in_configured_editor(
    settings: &AppSettings,
    path: &str,
    line: u32,
    column: u32,
) -> Result<(), String> {
    let candidate = Path::new(path);
    if !candidate.exists() {
        return Err(format!("Path does not exist: {path}"));
    }
    if !candidate.is_file() {
        return Err(format!(
            "Editor expects a file path, but got non-file path: {path}"
        ));
    }

    info!(
        "[open_result_in_editor] mode={:?} preset={:?} path={} line={} column={}",
        settings.editor.mode, settings.editor.preset, path, line, column
    );
    match settings.editor.mode {
        EditorMode::System => open_with_system_default(path),
        EditorMode::Preset => {
            let kind = resolve_editor_kind(&settings.editor.preset);
            Editor::builder()
                .file(path)
                .line(line)
                .column(column)
                .editor(kind)
                .open()
                .map_err(|e| format!("Failed to open in preset editor: {e}"))?;
            info!("[open_result_in_editor] preset editor launch requested successfully");
            Ok(())
        }
        EditorMode::CustomTemplate => {
            info!(
                "[open_result_in_editor] custom template={}",
                settings.editor.custom_template
            );
            let res =
                open_with_custom_template(&settings.editor.custom_template, path, line, column);
            if let Err(ref e) = res {
                error!(
                    "[open_result_in_editor] custom template launch failed: {}",
                    e
                );
            }
            res
        }
    }
}

fn build_pattern(params: &SearchParams) -> String {
    let base = if params.literal || !params.use_regex {
        regex::escape(&params.pattern)
    } else if params.use_regex {
        params.pattern.clone()
    } else {
        params.pattern.clone()
    };

    if params.whole_word {
        format!(r"\b(?:{})\b", base)
    } else {
        base
    }
}

fn build_walker(params: &SearchParams) -> Result<ignore::Walk, String> {
    let mut overrides = OverrideBuilder::new(&params.path);
    for include in &params.file_types {
        let include = include.trim();
        if include.is_empty() {
            continue;
        }
        overrides
            .add(include)
            .map_err(|e| format!("Invalid include pattern '{}': {e}", include))?;
    }
    for exclude in &params.exclude_patterns {
        let exclude = exclude.trim().trim_start_matches('!');
        if exclude.is_empty() {
            continue;
        }
        let pattern = format!("!{}", exclude);
        overrides
            .add(&pattern)
            .map_err(|e| format!("Invalid exclude pattern '{}': {e}", exclude))?;
    }

    let mut walker = WalkBuilder::new(&params.path);
    walker
        .overrides(
            overrides
                .build()
                .map_err(|e| format!("Failed to build glob overrides: {e}"))?,
        )
        .standard_filters(true)
        .max_depth(params.max_depth.map(|d| d as usize))
        .follow_links(false)
        .threads(1);

    Ok(walker.build())
}

fn collect_context_lines(
    path: &Path,
    line_number: u32,
    before: usize,
    after: usize,
) -> (Vec<String>, Vec<String>) {
    if before == 0 && after == 0 {
        return (Vec::new(), Vec::new());
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return (Vec::new(), Vec::new());
    };
    let lines: Vec<&str> = content.lines().collect();
    if line_number == 0 {
        return (Vec::new(), Vec::new());
    }

    let idx = (line_number as usize).saturating_sub(1);
    if idx >= lines.len() {
        return (Vec::new(), Vec::new());
    }

    let start = idx.saturating_sub(before);
    let end = (idx + after + 1).min(lines.len());

    let before_lines = lines[start..idx]
        .iter()
        .map(|line| (*line).to_string())
        .collect::<Vec<_>>();
    let after_lines = lines[idx + 1..end]
        .iter()
        .map(|line| (*line).to_string())
        .collect::<Vec<_>>();

    (before_lines, after_lines)
}

fn run_search_loop<M: Matcher>(
    walker: ignore::Walk,
    searcher: &mut grep_searcher::Searcher,
    matcher: &M,
    params: &SearchParams,
    pattern: &str,
    start_time: Instant,
    timeout: Duration,
    results: &mut Vec<SearchResult>,
    files_scanned: &mut usize,
    files_skipped: &mut usize,
) -> Result<(), String> {
    for entry in walker {
        if start_time.elapsed() > timeout {
            return Err(format!(
                "Search timeout: operation took longer than {} seconds",
                params.timeout_seconds
            ));
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                if err
                    .io_error()
                    .is_some_and(|io_err| io_err.kind() == ErrorKind::PermissionDenied)
                {
                    *files_skipped += 1;
                    continue;
                }
                return Err(format!("Failed to read search path entry: {err}"));
            }
        };

        if !entry
            .file_type()
            .map(|file_type| file_type.is_file())
            .unwrap_or(false)
        {
            continue;
        }

        let path = entry.into_path();
        *files_scanned += 1;
        let sink = Lossy(|line_number: u64, line: &str| {
            if start_time.elapsed() > timeout {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "search timeout"));
            }

            let line_number = line_number as u32;
            let (before_context, after_context) =
                collect_context_lines(&path, line_number, params.before_context, params.after_context);
            let raw_content = line.trim_end_matches(['\r', '\n']).to_string();
            let content = format_line_with_printer(pattern, params.case_sensitive, &raw_content)
                .unwrap_or(raw_content);
            results.push(SearchResult {
                path: path.display().to_string(),
                line_number,
                content,
                before_context,
                after_context,
            });

            Ok(results.len() < params.max_results)
        });

        if let Err(err) = searcher.search_path(matcher, &path, sink) {
            if err.kind() == io::ErrorKind::TimedOut {
                return Err(format!(
                    "Search timeout: operation took longer than {} seconds",
                    params.timeout_seconds
                ));
            }
            if err.kind() == io::ErrorKind::PermissionDenied {
                *files_skipped += 1;
                continue;
            }
            if err.kind() == io::ErrorKind::InvalidData {
                if matches!(params.binary_policy, BinaryPolicy::Skip | BinaryPolicy::Lossy) {
                    *files_skipped += 1;
                    continue;
                }
            }
            return Err(format!("Search failed for '{}': {err}", path.display()));
        }

        if results.len() >= params.max_results {
            break;
        }
    }

    Ok(())
}

fn format_line_with_printer(pattern: &str, case_sensitive: bool, line: &str) -> Option<String> {
    let mut matcher_builder = RegexMatcherBuilder::new();
    matcher_builder.case_insensitive(!case_sensitive);
    let matcher_for_search = matcher_builder.build(pattern).ok()?;
    let matcher_for_sink = matcher_builder.build(pattern).ok()?;

    let mut printer = StandardBuilder::new().build_no_color(Vec::<u8>::new());
    let mut searcher = SearcherBuilder::new().line_number(false).build();
    searcher
        .search_slice(
            &matcher_for_search,
            line.as_bytes(),
            printer.sink(matcher_for_sink),
        )
        .ok()?;
    let out = String::from_utf8(printer.into_inner().into_inner()).ok()?;
    let normalized = out.trim_end_matches(['\r', '\n']).to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

async fn search_with_ripgrep(params: SearchParams) -> Result<(Vec<SearchResult>, SearchRunStats), String> {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(params.timeout_seconds);
    if timeout.is_zero() {
        return Err("Search timeout: operation took longer than 0 seconds".to_string());
    }
    if params.max_results == 0 {
        return Ok((
            Vec::new(),
            SearchRunStats {
                elapsed_ms: 0,
                files_scanned: 0,
                files_skipped: 0,
            },
        ));
    }
    let pattern = build_pattern(&params);

    let mut searcher = SearcherBuilder::new()
        .line_number(true)
        .multi_line(params.multiline)
        .build();
    let mut results = Vec::new();
    let mut files_scanned = 0usize;
    let mut files_skipped = 0usize;
    match params.engine {
        SearchEngine::RustRegex => {
            let mut matcher_builder = RegexMatcherBuilder::new();
            matcher_builder.case_insensitive(!params.case_sensitive);
            let matcher = matcher_builder
                .build(&pattern)
                .map_err(|e| format!("Invalid search pattern: {e}"))?;
            let walker = build_walker(&params)?;
            run_search_loop(
                walker,
                &mut searcher,
                &matcher,
                &params,
                &pattern,
                start_time,
                timeout,
                &mut results,
                &mut files_scanned,
                &mut files_skipped,
            )?;
        }
        SearchEngine::Pcre2 => {
            #[cfg(feature = "pcre2")]
            {
                let mut matcher_builder = grep_pcre2::RegexMatcherBuilder::new();
                matcher_builder.caseless(!params.case_sensitive);
                let matcher = matcher_builder
                    .build(&pattern)
                    .map_err(|e| format!("Invalid PCRE2 pattern: {e}"))?;
                let walker = build_walker(&params)?;
                run_search_loop(
                    walker,
                    &mut searcher,
                    &matcher,
                    &params,
                    &pattern,
                    start_time,
                    timeout,
                    &mut results,
                    &mut files_scanned,
                    &mut files_skipped,
                )?;
            }
            #[cfg(not(feature = "pcre2"))]
            {
                return Err(
                    "PCRE2 engine requested, but binary was built without `pcre2` feature"
                        .to_string(),
                );
            }
        }
    }

    Ok((
        results,
        SearchRunStats {
            elapsed_ms: start_time.elapsed().as_millis(),
            files_scanned,
            files_skipped,
        },
    ))
}

pub async fn perform_search(params: SearchParams) -> Result<Vec<SearchResult>, String> {
    let (results, _stats) = search_with_ripgrep(params).await?;
    Ok(results)
}

fn page_from_items(
    items: &[SearchResult],
    cursor: usize,
    page_size: usize,
    stats: &SearchRunStats,
) -> SearchPage {
    let start = cursor.min(items.len());
    let end = (start + page_size).min(items.len());
    let page_items = items[start..end].to_vec();
    let returned_in_page = page_items.len();
    SearchPage {
        items: page_items,
        has_more: end < items.len(),
        next_cursor: (end < items.len()).then_some(end),
        total: items.len(),
        stats: SearchPageStats {
            total_matches: items.len(),
            returned_in_page,
            page_size,
            elapsed_ms: stats.elapsed_ms,
            files_scanned: stats.files_scanned,
            files_skipped: stats.files_skipped,
        },
    }
}

fn show_and_focus_quick_search(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("quick-search") else {
        return Err("Quick search window not found".to_string());
    };

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit_to("quick-search", "quick-search-activated", ());
    Ok(())
}

fn toggle_quick_search_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("quick-search") else {
        return Err("Quick search window not found".to_string());
    };

    if window.is_visible().map_err(|e| e.to_string())? {
        window.hide().map_err(|e| e.to_string())?;
        return Ok(());
    }

    show_and_focus_quick_search(app)
}

fn show_and_focus_settings(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("Settings window not found".to_string());
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

fn is_usable_active_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return false;
    }
    let candidate = Path::new(trimmed);
    candidate.exists() && candidate.is_dir()
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Option<String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(target_os = "macos")]
fn normalize_active_path(raw: &str) -> Option<String> {
    let mut path = raw.trim().to_string();
    if path.is_empty() {
        return None;
    }

    if let Some(stripped) = path.strip_prefix("file://") {
        // AXDocument often returns file:// URLs.
        path = stripped.replace("%20", " ");
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy().to_string();
        if path == "~" {
            path = home;
        } else if let Some(rest) = path.strip_prefix("~/") {
            path = format!("{home}/{rest}");
        }
    }

    let candidate = Path::new(&path);
    if candidate.is_file() {
        return candidate.parent().map(|p| p.to_string_lossy().to_string());
    }
    Some(path)
}

#[cfg(target_os = "macos")]
fn detect_frontmost_process() -> Option<(String, String)> {
    let pid = run_osascript(
        r#"tell application "System Events" to unix id of first process whose frontmost is true"#,
    )?;
    let name = run_osascript(
        r#"tell application "System Events" to name of first process whose frontmost is true"#,
    )?;
    Some((pid, name))
}

#[cfg(target_os = "macos")]
fn detect_from_frontmost_axdocument() -> Option<String> {
    // Uses accessibility metadata of frontmost app window (works for many editors/IDEs).
    let doc = run_osascript(
        r#"tell application "System Events" to value of attribute "AXDocument" of front window of (first process whose frontmost is true)"#,
    )?;
    let normalized = normalize_active_path(&doc)?;
    if is_usable_active_path(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn detect_from_lsof_cwd(pid: &str) -> Option<String> {
    let cwd_output = Command::new("lsof")
        .args(["-a", "-p", pid, "-d", "cwd", "-Fn"])
        .output()
        .ok()?;
    if !cwd_output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&cwd_output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            let normalized = normalize_active_path(path)?;
            if is_usable_active_path(&normalized) {
                return Some(normalized);
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn detect_from_finder() -> Option<String> {
    let finder = run_osascript(
        r#"tell application "Finder" to if (count of windows) > 0 then POSIX path of (target of front window as alias)"#,
    )?;
    let normalized = normalize_active_path(&finder)?;
    if is_usable_active_path(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn detect_active_path_platform() -> Option<String> {
    let (pid, process_name) = match detect_frontmost_process() {
        Some(data) => data,
        None => {
            warn!("[detect_active_path] source=none reason=frontmost_process_unavailable");
            return detect_from_finder();
        }
    };

    info!(
        "[detect_active_path] frontmost process='{}' pid={}",
        process_name, pid
    );

    // Cursor/VS Code family tend to expose a lot of internal storage via `lsof`,
    // so their `lsof`-based heuristics are usually noisy and hurt UX.
    // For these apps we prefer:
    // 1) AXDocument (if accessibility is available),
    // 2) Finder fallback.
    let pn = process_name.to_lowercase();
    let is_cursor_family = pn.contains("cursor")
        || pn.contains("vscode")
        || pn.contains("vscodium")
        || pn.contains("visual studio code")
        || pn == "code"
        || pn.contains("code");

    if is_cursor_family {
        if let Some(path) = detect_from_frontmost_axdocument() {
            info!(
                "[detect_active_path] source=cursor_axdocument kind=axdocument path={}",
                path
            );
            return Some(path);
        }

        info!(
            "[detect_active_path] source=cursor_axdocument kind=axdocument reason=empty_or_unusable_fallback_to_finder"
        );

        if let Some(path) = detect_from_finder() {
            info!(
                "[detect_active_path] source=cursor_finder kind=finder path={}",
                path
            );
            return Some(path);
        }

        warn!(
            "[detect_active_path] source=cursor_finder reason=finder_unavailable_or_unusable"
        );
        return None;
    }

    // 1) Frontmost application metadata (AXDocument), especially useful for IDEs/editors.
    if let Some(path) = detect_from_frontmost_axdocument() {
        info!(
            "[detect_active_path] source=frontmost_app_specific kind=axdocument path={}",
            path
        );
        return Some(path);
    } else {
        info!(
            "[detect_active_path] source=frontmost_app_specific kind=axdocument reason=empty_or_unusable"
        );
    }

    // 2) Fallback to frontmost process CWD.
    if let Some(path) = detect_from_lsof_cwd(&pid) {
        info!("[detect_active_path] source=lsof_cwd path={}", path);
        return Some(path);
    } else {
        info!("[detect_active_path] source=lsof_cwd reason=empty_or_unusable");
    }

    // 3) Fallback to Finder front window.
    if let Some(path) = detect_from_finder() {
        info!("[detect_active_path] source=finder path={}", path);
        return Some(path);
    }

    warn!("[detect_active_path] source=none reason=no_valid_path_found");
    None
}

#[cfg(target_os = "windows")]
fn detect_active_path_platform() -> Option<String> {
    let script = r#"
$shell = New-Object -ComObject Shell.Application
$window = $shell.Windows() | Where-Object { $_.Name -eq 'File Explorer' } | Select-Object -First 1
if ($window -and $window.Document -and $window.Document.Folder) { $window.Document.Folder.Self.Path }
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

#[cfg(target_os = "linux")]
fn detect_active_path_platform() -> Option<String> {
    for cmd in [
        ("sh", vec!["-c", "xdotool getwindowfocus getwindowname"]),
        ("sh", vec!["-c", "pwd"]),
    ] {
        let output = Command::new(cmd.0).args(cmd.1).output().ok()?;
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if out.starts_with('/') && !out.is_empty() {
                return Some(out);
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn detect_active_path_platform() -> Option<String> {
    None
}

#[tauri::command]
async fn detect_active_path() -> Result<Option<String>, String> {
    Ok(detect_active_path_platform())
}

#[tauri::command]
async fn hide_quick_search_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("quick-search") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn show_quick_search_window(app: AppHandle) -> Result<(), String> {
    show_and_focus_quick_search(&app)
}

#[tauri::command]
async fn show_settings_window(app: AppHandle) -> Result<(), String> {
    show_and_focus_settings(&app)
}

#[tauri::command]
async fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| format!("Failed to get autostart state: {e}"))
}

#[tauri::command]
async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let state = app.state::<RuntimeState>();
    state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())
        .map(|s| s.clone())
}

#[tauri::command]
async fn set_autostart_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {e}"))?;
    } else {
        manager
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {e}"))?;
    }

    {
        let state = app.state::<RuntimeState>();
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| "Settings lock poisoned".to_string())?;
        settings.autostart = enabled;
        save_settings(&app, &settings)?;
    }
    {
        let state = app.state::<RuntimeState>();
        if let Ok(guard) = state.autostart_menu_item.lock() {
            if let Some(item) = guard.as_ref() {
                let _ = item.set_checked(enabled);
            }
        };
    }

    Ok(())
}

#[tauri::command]
async fn update_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let state = app.state::<RuntimeState>();
    let previous_hotkey = state
        .current_hotkey
        .lock()
        .map_err(|_| "Hotkey lock poisoned".to_string())?
        .clone();

    apply_hotkey(&app, &previous_hotkey, &settings.hotkey)?;

    if settings.autostart {
        let _ = app.autolaunch().enable();
    } else {
        let _ = app.autolaunch().disable();
    }

    {
        let mut current_hotkey = state
            .current_hotkey
            .lock()
            .map_err(|_| "Hotkey lock poisoned".to_string())?;
        *current_hotkey = settings.hotkey.clone();
    }
    {
        let mut stored = state
            .settings
            .lock()
            .map_err(|_| "Settings lock poisoned".to_string())?;
        *stored = settings.clone();
        save_settings(&app, &stored)?;
        if let Ok(guard) = state.autostart_menu_item.lock() {
            if let Some(item) = guard.as_ref() {
                let _ = item.set_checked(settings.autostart);
            }
        }
    }

    Ok(settings)
}

#[tauri::command]
async fn open_result_in_editor(
    app: AppHandle,
    path: String,
    line: u32,
    column: Option<u32>,
) -> Result<(), String> {
    let state = app.state::<RuntimeState>();
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?
        .clone();
    open_in_configured_editor(&settings, &path, line, column.unwrap_or(1))
}

#[tauri::command]
async fn reveal_in_file_manager(path: String) -> Result<(), String> {
    let candidate = Path::new(&path);
    let reveal_path = if candidate.is_file() {
        candidate.parent().unwrap_or(candidate)
    } else {
        candidate
    };
    let reveal = reveal_path.to_string_lossy().to_string();
    if reveal.trim().is_empty() {
        return Err("Path is empty".to_string());
    }
    open_with_system_default(&reveal)
}

#[tauri::command]
async fn open_terminal_at_path(path: String) -> Result<(), String> {
    let candidate = Path::new(&path);
    let target = if candidate.is_file() {
        candidate.parent().unwrap_or(candidate)
    } else {
        candidate
    };
    let dir = target.to_string_lossy().to_string();
    if dir.trim().is_empty() {
        return Err("Path is empty".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "Terminal", &dir])
            .spawn()
            .map_err(|e| format!("Failed to open Terminal: {e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("Failed to open terminal directory: {e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "wt.exe", "-d", &dir])
            .spawn()
            .map_err(|e| format!("Failed to open Windows Terminal: {e}"))?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("Open terminal is not supported on this platform".to_string())
}

#[tauri::command]
async fn test_open_in_editor(app: AppHandle) -> Result<String, String> {
    let state = app.state::<RuntimeState>();
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?
        .clone();

    let demo_dir = std::env::temp_dir().join("search-bolt");
    fs::create_dir_all(&demo_dir).map_err(|e| {
        format!(
            "Failed to create demo directory '{}': {e}",
            demo_dir.display()
        )
    })?;

    let demo_file = demo_dir.join("editor-test.txt");
    let content = [
        "search-bolt editor test file",
        "",
        "If your editor opened this file, launch settings work correctly.",
        "Line 4: quick test marker",
    ]
    .join("\n");
    fs::write(&demo_file, content)
        .map_err(|e| format!("Failed to write demo file '{}': {e}", demo_file.display()))?;

    let demo_path = demo_file.to_string_lossy().to_string();
    info!("[test_open_in_editor] opening demo file: {}", demo_path);
    open_in_configured_editor(&settings, &demo_path, 4, 1)?;
    Ok(demo_path)
}

#[tauri::command]
async fn search_start(app: AppHandle, params: SearchParams) -> Result<SearchStartResponse, String> {
    let (results, run_stats) = search_with_ripgrep(params.clone()).await?;
    let page_size = params.page_size.max(1);
    let session_id = uuid::Uuid::new_v4().to_string();
    let page = page_from_items(&results, 0, page_size, &run_stats);

    let state = app.state::<RuntimeState>();
    state
        .search_sessions
        .lock()
        .map_err(|_| "Search sessions lock poisoned".to_string())?
        .insert(
            session_id.clone(),
            SearchSession {
                items: results,
                page_size,
                elapsed_ms: run_stats.elapsed_ms,
                files_scanned: run_stats.files_scanned,
                files_skipped: run_stats.files_skipped,
            },
        );

    Ok(SearchStartResponse { session_id, page })
}

#[tauri::command]
async fn search_next(
    app: AppHandle,
    session_id: String,
    cursor: Option<usize>,
) -> Result<SearchPage, String> {
    let state = app.state::<RuntimeState>();
    let sessions = state
        .search_sessions
        .lock()
        .map_err(|_| "Search sessions lock poisoned".to_string())?;
    let Some(session) = sessions.get(&session_id) else {
        return Err(format!("Search session not found: {session_id}"));
    };

    Ok(page_from_items(
        &session.items,
        cursor.unwrap_or(0),
        session.page_size,
        &SearchRunStats {
            elapsed_ms: session.elapsed_ms,
            files_scanned: session.files_scanned,
            files_skipped: session.files_skipped,
        },
    ))
}

#[tauri::command]
async fn search_cancel(app: AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<RuntimeState>();
    state
        .search_sessions
        .lock()
        .map_err(|_| "Search sessions lock poisoned".to_string())?
        .remove(&session_id);
    Ok(())
}

#[tauri::command]
async fn search(app: AppHandle, params: SearchParams) -> Result<Vec<SearchResult>, String> {
    let response = search_start(app, params).await?;
    Ok(response.page.items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(Target::new(TargetKind::Stdout))
                .target(Target::new(TargetKind::Webview))
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                // Tray-first UX: hide Dock icon and app presence in the macOS Dock.
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let mut settings = load_settings(&app.handle());

            if settings.autostart {
                let _ = app.autolaunch().enable();
            } else {
                let _ = app.autolaunch().disable();
            }

            // Keep settings window hidden by default on startup.
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.hide();
            }

            let quick_label = "quick-search";
            if app.get_webview_window(quick_label).is_none() {
                WebviewWindowBuilder::new(
                    app,
                    quick_label,
                    WebviewUrl::App("index.html?quick=1".into()),
                )
                .title("Quick Search")
                .visible(false)
                .decorations(false)
                .resizable(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .inner_size(
                    settings.quick_window_sizes.width as f64,
                    settings.quick_window_sizes.idle_height as f64,
                )
                .build()
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
            }

            app.manage(RuntimeState {
                settings: Mutex::new(settings.clone()),
                current_hotkey: Mutex::new(settings.hotkey.clone()),
                autostart_menu_item: Mutex::new(None),
                search_sessions: Mutex::new(HashMap::new()),
            });

            // Tray icon with quick actions.
            let open_search =
                MenuItemBuilder::with_id("tray-open-search", "Open Search").build(app)?;
            let open_settings =
                MenuItemBuilder::with_id("tray-open-settings", "Settings").build(app)?;
            let autostart = CheckMenuItemBuilder::with_id("tray-autostart", "Autostart")
                .checked(settings.autostart)
                .build(app)?;
            let quit = MenuItemBuilder::with_id("tray-quit", "Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&open_search)
                .item(&open_settings)
                .separator()
                .item(&autostart)
                .separator()
                .item(&quit)
                .build()?;

            let autostart_item = autostart.clone();
            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
                .menu(&tray_menu)
                .show_menu_on_left_click(settings.tray.menu_on_left_click)
                .on_menu_event(move |app, event| match event.id().0.as_str() {
                    "tray-open-search" => {
                        let _ = show_and_focus_quick_search(app);
                    }
                    "tray-open-settings" => {
                        let _ = show_and_focus_settings(app);
                    }
                    "tray-autostart" => {
                        let next = !app.autolaunch().is_enabled().unwrap_or(false);
                        if next {
                            let _ = app.autolaunch().enable();
                        } else {
                            let _ = app.autolaunch().disable();
                        }
                        let _ = autostart_item.set_checked(next);
                        if let Some(state) = app.try_state::<RuntimeState>() {
                            if let Ok(mut stored) = state.settings.lock() {
                                stored.autostart = next;
                                let _ = save_settings(app, &stored);
                            }
                            if let Ok(guard) = state.autostart_menu_item.lock() {
                                if let Some(item) = guard.as_ref() {
                                    let _ = item.set_checked(next);
                                }
                            };
                        }
                    }
                    "tray-quit" => {
                        app.exit(0);
                    }
                    _ => {}
                });

            if let Some(icon) = app.default_window_icon().cloned() {
                tray_builder = tray_builder.icon(icon);
            }
            let tray = tray_builder.build(app)?;
            // Keep tray icon alive for the app lifetime.
            app.manage(tray);
            {
                let state = app.state::<RuntimeState>();
                if let Ok(mut slot) = state.autostart_menu_item.lock() {
                    *slot = Some(autostart.clone());
                };
            }

            #[cfg(desktop)]
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, _shortcut, event| {
                            if event.state() == ShortcutState::Pressed {
                                let _ = toggle_quick_search_window(app);
                            }
                        })
                        .build(),
                )?;
                if settings.hotkey.trim().is_empty() {
                    settings.hotkey = AppSettings::default().hotkey;
                }
                app.global_shortcut()
                    .register(settings.hotkey.as_str())
                    .map_err(|e| -> Box<dyn std::error::Error> {
                        format!("Failed to register hotkey '{}': {e}", settings.hotkey).into()
                    })?;
            }

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            search,
            search_start,
            search_next,
            search_cancel,
            open_result_in_editor,
            reveal_in_file_manager,
            open_terminal_at_path,
            test_open_in_editor,
            hide_quick_search_window,
            show_quick_search_window,
            show_settings_window,
            get_settings,
            update_settings,
            is_autostart_enabled,
            set_autostart_enabled,
            detect_active_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(path, content).expect("write file");
    }

    fn base_params(root: &std::path::Path) -> SearchParams {
        SearchParams {
            path: root.display().to_string(),
            pattern: "needle".to_string(),
            case_sensitive: true,
            whole_word: false,
            use_regex: true,
            literal: false,
            multiline: false,
            before_context: 0,
            after_context: 0,
            engine: SearchEngine::RustRegex,
            binary_policy: BinaryPolicy::Lossy,
            max_depth: None,
            file_types: Vec::new(),
            exclude_patterns: Vec::new(),
            page_size: 50,
            max_results: 100,
            timeout_seconds: 5,
        }
    }

    #[test]
    fn whole_word_matches_only_full_words() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("sample.txt");
        write_file(&file, "needle\nneedleman\nsuperneedle\nneedle\n");

        let mut params = base_params(dir.path());
        params.whole_word = true;
        params.pattern = "needle".to_string();
        params.use_regex = false;

        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.content == "needle"));
    }

    #[test]
    fn literal_mode_escapes_regex_metacharacters() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("regex.txt");
        write_file(&file, "a.c\nabc\n");

        let mut params = base_params(dir.path());
        params.pattern = "a.c".to_string();
        params.use_regex = false;

        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "a.c");
    }

    #[test]
    fn max_depth_limits_recursion() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("root.txt"), "needle\n");
        write_file(&dir.path().join("nested/deep.txt"), "needle\n");

        let mut params = base_params(dir.path());
        params.max_depth = Some(1);

        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("root.txt"));
    }

    #[test]
    fn include_and_exclude_globs_match_rg_semantics() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("a.keep.rs"), "needle\n");
        write_file(&dir.path().join("b.skip.rs"), "needle\n");
        write_file(&dir.path().join("c.txt"), "needle\n");

        let mut params = base_params(dir.path());
        params.file_types = vec!["*.rs".to_string()];
        params.exclude_patterns = vec!["*skip*".to_string()];

        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("a.keep.rs"));
    }

    #[test]
    fn context_lines_are_collected() {
        let dir = TempDir::new().expect("temp dir");
        write_file(
            &dir.path().join("ctx.txt"),
            "line1\nline2\nneedle\nline4\nline5\n",
        );
        let mut params = base_params(dir.path());
        params.before_context = 1;
        params.after_context = 2;

        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].before_context, vec!["line2".to_string()]);
        assert_eq!(
            results[0].after_context,
            vec!["line4".to_string(), "line5".to_string()]
        );
    }

    #[test]
    fn page_from_items_paginates() {
        let items = vec![
            SearchResult {
                path: "a".to_string(),
                line_number: 1,
                content: "one".to_string(),
                before_context: Vec::new(),
                after_context: Vec::new(),
            },
            SearchResult {
                path: "b".to_string(),
                line_number: 2,
                content: "two".to_string(),
                before_context: Vec::new(),
                after_context: Vec::new(),
            },
            SearchResult {
                path: "c".to_string(),
                line_number: 3,
                content: "three".to_string(),
                before_context: Vec::new(),
                after_context: Vec::new(),
            },
        ];
        let stats = SearchRunStats {
            elapsed_ms: 1,
            files_scanned: 3,
            files_skipped: 0,
        };
        let page1 = page_from_items(&items, 0, 2, &stats);
        let page2 = page_from_items(&items, page1.next_cursor.unwrap_or(0), 2, &stats);
        assert_eq!(page2.items.len(), 1);
        assert!(!page2.has_more);
        assert_eq!(page2.next_cursor, None);
    }

    #[test]
    fn timeout_zero_fails_fast() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("big.txt"), &"needle\n".repeat(10_000));

        let mut params = base_params(dir.path());
        params.timeout_seconds = 0;

        let err = tauri::async_runtime::block_on(perform_search(params)).expect_err("must timeout");
        assert!(err.contains("Search timeout"));
    }

    #[test]
    fn max_results_zero_returns_empty() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("any.txt"), "needle\nneedle\n");

        let mut params = base_params(dir.path());
        params.max_results = 0;

        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert!(results.is_empty());
    }

    #[test]
    fn binary_invalid_utf8_file_is_skipped() {
        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("good.txt"), "needle\n");

        let mut binary = fs::File::create(dir.path().join("bad.lib")).expect("create binary");
        binary
            .write_all(&[0xff, 0xfe, 0xfd, b'n', b'e', b'e', b'd', b'l', b'e'])
            .expect("write binary");

        let params = base_params(dir.path());
        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("good.txt"));
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_file_is_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        write_file(&dir.path().join("good.txt"), "needle\n");
        let restricted = dir.path().join("private.txt");
        write_file(&restricted, "needle\n");

        let mut perms = fs::metadata(&restricted).expect("metadata").permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&restricted, perms).expect("set permissions");

        let params = base_params(dir.path());
        let results = tauri::async_runtime::block_on(perform_search(params)).expect("search ok");
        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("good.txt"));

        // Restore permissions so tempfile cleanup works reliably.
        let mut restore = fs::metadata(&restricted).expect("metadata").permissions();
        restore.set_mode(0o600);
        fs::set_permissions(&restricted, restore).expect("restore permissions");
    }
}

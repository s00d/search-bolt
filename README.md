[![Release](https://img.shields.io/badge/Download-Release-000081?style=for-the-badge)](https://github.com/s00d/search-bolt/releases)
[![GitHub issues](https://img.shields.io/github/issues/s00d/search-bolt?style=for-the-badge)](https://github.com/s00d/search-bolt/issues)
[![GitHub stars](https://img.shields.io/github/stars/s00d/search-bolt?style=for-the-badge)](https://github.com/s00d/search-bolt/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

# Search Bolt

Search Bolt is a hotkey-first desktop search tool for code and text files.
It uses Tauri + Vue on the UI side and the ripgrep Rust crates directly in-process (no shelling out to external `rg` commands at runtime).

The app is designed around a quick-search popup: press a global shortcut anywhere, type your query, get results, open a file at the exact line.

![Search Bolt Screenshot](https://github.com/s00d/search-bolt/blob/main/public/Screenshot.png?raw=true)

## Core Approach

- **Hotkey-first workflow**: open the quick-search window globally from any app.
- **Tray-oriented behavior**: app can run in the tray while search opens on demand.
- **Native ripgrep pipeline**: backend uses ripgrep ecosystem crates directly from Rust.
- **Chunked search sessions**: results are streamed in pages (`search_start` / `search_next` / `search_cancel`) for responsive UI.
- **Editor integration**: open matches in configured editors at line/column.
- **Persistent settings**: all settings are stored in a local key-value store.

## How It Works

1. Press the configured global hotkey.
2. Search Bolt opens a compact quick-search window (without standard frame controls).
3. The app tries to prefill a useful search path from the active context (with platform fallbacks).
4. Submit a query; backend starts a search session and returns the first page.
5. Load more pages when needed, without freezing the UI.
6. Open any result in your preferred editor at the matched line.

## Search Features

- Regex and literal modes
- Case-sensitive option
- Whole-word matching
- Multiline support
- Before/after context lines
- Include/exclude filters
- File type filtering
- Binary handling policy (`skip` or `lossy`)
- Engine selection (`rust_regex`, optional `pcre2` build)
- Match highlighting for global pattern and local filter
- Search statistics (elapsed time, scanned/skipped files, page counters)

## Editor Support

Search Bolt supports default OS opener and configurable editor launch commands/presets.
This includes common editors such as:

- VS Code / VS Code Insiders / VSCodium / Cursor / Windsurf
- Sublime Text / Zed / Helix / Vim / NeoVim / Emacs / Nano
- JetBrains IDEs (line-based launch where supported)
- Xcode / TextMate / Notepad++

## Tech Stack

- [Tauri 2](https://tauri.app/)
- [Vue 3](https://vuejs.org/)
- [Tailwind CSS 4](https://tailwindcss.com/)
- [ripgrep crates](https://github.com/BurntSushi/ripgrep): `ignore`, `grep-regex`, `grep-searcher`, `grep-matcher`, `grep-printer`, `grep`, optional `grep-pcre2`
- Tauri plugins: `global-shortcut`, `autostart`, `store`, `log`, `dialog`, `opener`

## Requirements

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/)
- [Rust stable](https://www.rust-lang.org/tools/install)

## Development

```bash
git clone https://github.com/s00d/search-bolt
cd search-bolt
pnpm install
pnpm tauri dev
```

## Production Build

```bash
pnpm tauri build
```

Artifacts are generated under `src-tauri/target/`.

## Release Workflow

The repository includes a GitHub Actions release workflow that builds:

- macOS universal (`universal-apple-darwin`)
- Linux
- Windows (x64 and i686)

## Configuration Notes

- Global hotkey, autostart, search defaults, editor settings, and behavior options are managed in-app and persisted.
- Logging is provided through `tauri-plugin-log` for both Rust and frontend diagnostics.
- On macOS, active path detection uses app-aware strategies and Finder fallback.

### mac OS Sign

```bash
chmod +x /Applications/search-bolt.app
xattr -cr /Applications/search-bolt.app
codesign --force --deep --sign - /Applications/search-bolt.app
```


## License

MIT

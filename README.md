[![Release](https://img.shields.io/badge/Download-Release-000081?style=for-the-badge)](https://github.com/s00d/search-bolt/releases/tag/v0.1.3)
[![GitHub issues](https://img.shields.io/github/issues/s00d/tauri-plugin-serialplugin?style=for-the-badge)](https://github.com/s00d/search-bolt/issues)
[![GitHub stars](https://img.shields.io/github/stars/s00d/tauri-plugin-serialplugin?style=for-the-badge)](https://github.com/s00d/search-bolt/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

# search bolt

A modern, visual search tool built with Tauri 2, Vue 3, and Tailwind CSS that provides a user-friendly interface for ripgrep or grep, the blazingly fast search tool.

![SearchBolt Screenshot](https://github.com/s00d/search-bolt/blob/main/public/Screenshot.png?raw=true)


## Features

- 🔍 Fast file content search using ripgrep
- 🎯 Advanced search options:
    - Case sensitivity toggle
    - Whole word matching
    - Regular expression support
    - Maximum search depth control
    - File type filtering
    - Pattern exclusion
- 💅 Syntax-highlighted search results
- 📋 One-click result copying
- 🎨 Clean, modern UI with Tailwind CSS
- 🚀 Cross-platform support (Windows, macOS, Linux)

## Prerequisites

- [Node.js](https://nodejs.org/) (v16 or later)
- [Rust](https://www.rust-lang.org/tools/install)
- [ripgrep](https://github.com/BurntSushi/ripgrep#installation) installed on your system

## Development Setup

1. Clone the repository:
```bash
git clone https://github.com/s00d/search-bolt
cd search-bolt
```

2. Install dependencies:
```bash
npm install
```

3. Start the development server:
```bash
npm run tauri dev
```

## Building for Production

To create a production build:

```bash
npm run tauri build
```

The built application will be available in the `src-tauri/target/release` directory.

## Usage

1. Launch the application
2. Click "Browse" to select a directory to search in
3. Enter your search pattern
4. Configure advanced options if needed:
    - Toggle case sensitivity
    - Enable whole word matching
    - Enable regex mode
    - Set maximum search depth
    - Add file type filters
    - Add exclusion patterns
5. Click "Search" to start searching
6. View results with syntax highlighting
7. Copy results to clipboard with one click

## Tech Stack

- [Tauri 2](https://tauri.app/) - Desktop application framework
- [Vue 3](https://vuejs.org/) - Frontend framework
- [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS framework
- [ripgrep](https://github.com/BurntSushi/ripgrep) - Search tool
- [highlight.js](https://highlightjs.org/) - Syntax highlighting
- [HeadlessUI](https://headlessui.dev/) - Unstyled UI components

## License

MIT

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request


### mac OS Sign

```bash
chmod +x /Applications/search-bolt.app
xattr -cr /Applications/search-bolt.app
codesign --force --deep --sign - /Applications/search-bolt.app
```

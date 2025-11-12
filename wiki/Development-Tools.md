# Development Tools

Additional development packages installed via npm, Cargo, and uv.

## npm Global Packages

8 JavaScript/TypeScript packages installed globally.

| Package | Version | Description |
|---------|---------|-------------|
| @github/copilot | 0.0.339 | GitHub Copilot CLI |
| @google/gemini-cli | 0.8.2 | Google Gemini CLI |
| @google/jules | 0.1.33 | Jules AI assistant |
| @openai/codex | 0.46.0 | OpenAI Codex CLI |
| @sourcegraph/amp | 0.0.1760127338-g8601e4 | Sourcegraph Amp |
| happy-coder | 0.11.1 | Coding assistant |
| vscode-langservers-extracted | 4.10.0 | VS Code language servers |
| @randomlabs/slatecli | 0.0.18 | Slate CLI |

### Installation

```bash
cat npm-globals.txt | grep -v "^#" | xargs npm install -g
```

### Update List

```bash
npm list -g --depth=0 | tail -n +2 | sed 's/^[├└]─* //' > npm-globals.txt
```

## Cargo Packages

5 Rust binaries installed via Cargo.

| Package | Description |
|---------|-------------|
| basalt-tui | TUI framework |
| bevy_debugger_mcp | Bevy game engine debugger MCP server |
| cargo-watch | Auto-rebuild on file changes |
| repo2prompt | Convert repositories to LLM prompts |
| ttyper | Terminal typing test game |

### Installation

```bash
cat cargo-installs.txt | grep -v "^#" | xargs cargo install
```

### Update List

```bash
ls ~/.cargo/bin/ | grep -v "^cargo\|^rust" > cargo-installs.txt
```

## uv Tools

3 Python tools installed via uv.

| Package | Description |
|---------|-------------|
| craft-cli | Craft CLI tool |
| posting | API testing tool |
| writekit-cli | Writing toolkit |

### Installation

```bash
cat uv-tools.txt | grep -v "^#" | xargs -I {} uv tool install {}
```

### Update List

```bash
uv tool list | grep -E "^\S" | awk '{print $1}' > uv-tools.txt
```

## Notes

- npm packages require Node.js (installed separately or via nvm/fnm)
- Cargo packages require Rust toolchain (see [Homebrew Packages](Homebrew-Packages))
- uv tools are isolated Python environments
- Package versions shown are current as of last update

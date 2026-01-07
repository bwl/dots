# Pake Desktop App Wrappers

Lightweight native desktop apps from websites using [Pake](https://github.com/tw93/Pake).

## Usage

```bash
# Build all apps
./build.sh

# Build single app
./build.sh --app X
./build.sh --app GitHub
```

Apps output to `~/Applications/Pake/`.

## Adding New Apps

1. Add entry to `Pakefile`:
   ```bash
   pake "https://example.com" "AppName" --width 1200 --height 800
   ```

2. Create inject files (optional):
   ```bash
   mkdir -p apps/example.com
   touch apps/example.com/inject.{js,css}
   ```

3. Run `./build.sh --app AppName`

## Structure

```
pake/
├── Pakefile              # App manifest
├── build.sh              # Build script
├── lib/
│   └── harness.js        # Shared inject code
└── apps/
    └── <domain>/
        ├── inject.js     # Site-specific JS
        └── inject.css    # Site-specific CSS
```

## Pakefile Format

```bash
pake "URL" "Name" [pake-cli options]
```

Options: `--width`, `--height`, `--hide-title-bar`, `--fullscreen`, etc.

## Inject Harness

`lib/harness.js` loads before per-site code. Provides:
- `PAKE.config` - site configuration object
- `PAKE.showHelp()` - stub for future which-key feature (Ctrl+?)

## Requirements

- [pake-cli](https://github.com/tw93/Pake): `npm install -g pake-cli`
- Rust toolchain (for building)

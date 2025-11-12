# Mac App Store Applications

10 GUI applications installed via Mac App Store.

## Applications

| App ID | Name | Version | Category |
|--------|------|---------|----------|
| 6448019325 | Apollo | 1.1.8 | Social Media |
| 1352778147 | Bitwarden | 2025.9.0 | Password Manager |
| 1236045954 | Canary Mail | 4.99.0 | Email Client |
| 1099568401 | Home Assistant | 2025.10.0 | Home Automation |
| 409183694 | Keynote | 14.4 | Presentations |
| 6446206067 | Klack | 1.7.2 | Utilities |
| 409203825 | Numbers | 14.4 | Spreadsheets |
| 1542143627 | OmniFocus | 4.8.4 | Task Management |
| 409201541 | Pages | 14.4 | Word Processing |
| 904280696 | Things | 3.22.5 | Task Management |

## Categories

**Productivity & Organization:**
- OmniFocus - GTD task manager
- Things - Task management
- Keynote - Presentations
- Numbers - Spreadsheets
- Pages - Word processing

**Communication:**
- Canary Mail - Email client
- Apollo - Reddit client

**Security & Automation:**
- Bitwarden - Password management
- Home Assistant - Home automation

**Utilities:**
- Klack - Keyboard sound effects

## Installation

All apps require Mac App Store login:

```bash
# Install all from Masfile
cat Masfile | awk '{print $1}' | xargs mas install

# Or install individually
mas install <app-id>
```

## Update List

```bash
mas list > Masfile
```

## Notes

- Version numbers may be outdated after updates
- Some apps require separate login (Bitwarden, Home Assistant)
- iWork apps (Keynote, Numbers, Pages) free with macOS

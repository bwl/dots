# Pake Next Steps

## Immediate
- [ ] Test build: `./build.sh` - verify apps build and launch
- [ ] Add to Brewfile: `pake-cli` (if not already)

## Which-Key Feature
- [ ] Research: How to detect site's registered keyboard event listeners
- [ ] Research: Scrape site's native help overlay (GitHub: `?`, X: `?`)
- [ ] Design: Unified overlay UI (modal with searchable shortcut list)
- [ ] Implement: Per-site poller to discover shortcuts dynamically
- [ ] Implement: Custom shortcut registration API (`PAKE.registerShortcut()`)

## CSS Tweaks (per-site)
- [ ] X.com: Hide Premium upsells, cleaner sidebar
- [ ] GitHub: Wider code view, cleaner PR interface

## Infrastructure
- [ ] Custom icons (v2): `apps/<domain>/icon.icns`
- [ ] Auto-update check for pake-cli
- [ ] Add more apps as needed (Claude, ChatGPT, Linear, etc.)

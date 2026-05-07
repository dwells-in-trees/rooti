# 🌳 Rooti
 
A biologically-inspired tree simulation that grows on at transparent, click-through overlay on your Windows desktop.
 
> **Status:** Early development (`0.x`) - expect breaking changes between versions.
 
---
 
## What is this?
 
`Rooti` is a living desktop ornament. A procedural tree grows organically in the background of you screen, simulating real biological processes - branching, suppression of nodes with auxin, gravitropism - while remaining fully transparent and non-intrusive. You can click through it, work normally, and just glance over occasionally to see how your tree is doing.

Inspired by the desktop pets of yester-year, this project serves no functional purpose and is fully aesthetic, ornamental, and just for fun!

Ultimately, this is a passion project to help the developer learn Rust!
 
---
 
## Features
 
- **Biological growth simulation** - branching driven by an auxin suppression model, producing emergent apical dominance and organic-looking structure
- **Transparent overlay** - renders over your desktop with a fully click-through window; no taskbar presence, no interruptions
- **Live parameter tuning** - settings panel to adjust growth rate, branching behaviour, gravitropism, and more
- **Ginkgo leaf rendering** - stylised leaf shapes with petiole placement; More tree types to come in the future!
 
---
 
## Platform support
 
| Platform | Status |
|---|---|
| Windows 10 / 11 | ✅ Primary target |
| macOS | 🔲 Planned |
| Linux | ❌ Not planned |
 
---
 
The release binary is a single self-contained `.exe` with no runtime dependencies — copy and run.
 
---
 
## Versioning & releases
 
This project follows [Semantic Versioning](https://semver.org/) starting at `0.x.y`. Commits follow the [Conventional Commits](https://www.conventionalcommits.org/) spec. Changelogs are generated with [`git-cliff`](https://git-cliff.org/).
 
---
 
## Licence
 
GPL-3 — see [`LICENSE`](LICENSE).

# Changelog

## [v0.3.0] - 2026-05-14

### Added

- Add deterministic random growth with seed values

## [v0.2.2] - 2026-05-07

### Added

- Add help section to menu bar with slint attributional widget *(ui)*
- Add GPL-3 LICENSE and edit cargo.toml to reflect

### Changed

- Scaffold files for modularization of main.rs
- Split main.rs into three modules
- Implement LCRS tree model to reduce unnecessary vec allocations in parent-child relationships *(tree)*

### Misc

- Update gitignore release-builds directory
- Update gitignore
- Remove code-workspace files and adjust git ignore

## [v0.2.1] - 2026-04-09

### Changed

- Adjust window configuration to hide main canvas taskbar icon

### Fixed

- Limit gravitropism threshold to 89 degrees in menu to prevent crash *(ui)*
- Limit gravitropism threshold to 89 degrees in menu to prevent crash *(ui)*

### Misc

- Add git cliff for changelog generation
- Release v0.2.1

## [v0.2.0] - 2026-04-08

### Added

- Add branching logic to tree growth
- Implement primitive thickness logic
- Add initial auxin simulation
- Add support for gravitropism
- Add functionality for pipe model thickness
- Add probability for nodes to activate
- Extend leaf functionality
- Add TreeConfig struct in support of debug menu
- Add debug menu for live edits to tree configuration

### Fixed

- Fix incorrect branch thickness *(ui)*

### Misc

- Change .gitignore
- Removed unnecessary files and updated .gitignore
- Add subsystem tag to suppress terminal at runtime
- Increment version to v0.2.0


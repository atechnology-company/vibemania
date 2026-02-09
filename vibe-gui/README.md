# VibeMania GUI

A native macOS application for managing VibeMania agents. Run multiple AI coding agents in parallel on the same project or across separate projects.

## Requirements

- macOS 14.0 (Sonoma) or later
- [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)
- Xcode 15+
- One or more AI coding tools installed:
  - [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (default)
  - [Amp CLI](https://ampcode.com)

## Setup

### 1. Generate the Xcode project

```bash
cd vibe-gui
xcodegen generate
```

This reads `project.yml` and produces `VibeMania.xcodeproj`.

### 2. Open in Xcode

```bash
open VibeMania.xcodeproj
```

### 3. Build & Run

Press **Cmd+R** in Xcode or build from the command line:

```bash
xcodebuild -project VibeMania.xcodeproj -scheme VibeMania -configuration Debug build
```

## Features

### Project Management
- Add projects by browsing to any local directory
- Configure tool type per project: **VibeMania (Claude)** or **VibeMania (Amp)**
- Set max iterations per project
- Edit or remove projects at any time

### Parallel Agent Execution
- Launch multiple agents on the same project simultaneously
- Run agents across different projects in parallel
- Each agent runs in its own process with independent output

### Real-Time Monitoring
- Live log streaming from each running agent
- Iteration progress tracking with visual progress bars
- Agent status indicators (running, completed, failed, stopped)
- Duration tracking for each agent session

### Dashboard
- Overview of all projects and agents
- Stats for running agents, total agents, and completed sessions
- Quick access to active and recent agent runs

## Architecture

```
Sources/VibeMania/
├── VibeManiaApp.swift          # App entry point (@main)
├── Models/
│   ├── Project.swift           # Project data model
│   ├── Agent.swift             # Agent state model (@Observable)
│   ├── AgentManager.swift      # Process lifecycle manager (@Observable)
│   └── ProjectStore.swift      # JSON persistence to ~/Library/Application Support/
└── Views/
    ├── ContentView.swift       # NavigationSplitView layout
    ├── SidebarView.swift       # Project list sidebar
    ├── DashboardView.swift     # Overview dashboard with stats
    ├── ProjectDetailView.swift # Project detail with agent list + settings
    ├── AgentCardView.swift     # Agent status card
    ├── LogView.swift           # Real-time monospaced log viewer
    └── AddProjectSheet.swift   # Add project sheet with directory picker
```

### Key Design Decisions

- **SwiftUI + `@Observable`** -- Modern reactive state management (macOS 14+)
- **`NavigationSplitView`** -- Native sidebar + detail layout
- **`Process` / `Pipe`** -- Spawns `vibemania.sh` as child processes
- **No sandbox** -- Required for file system access and process spawning
- **XcodeGen** -- Generates `.xcodeproj` from `project.yml`; the Xcode project is gitignored
- **JSON persistence** -- Projects saved to `~/Library/Application Support/VibeMania/`

## How It Works

1. **Add a project** -- Point VibeMania at any project directory
2. **Choose a tool** -- Select VibeMania (Claude) or VibeMania (Amp)
3. **Launch agents** -- Click "Launch Agent" to start an autonomous coding session
4. **Monitor progress** -- Watch real-time output in the log viewer
5. **Run in parallel** -- Launch additional agents on the same or different projects
6. **Stop when done** -- Stop individual agents or all agents for a project

The app finds `vibemania.sh` in your project directory (or common install locations) and runs it with the `--tool` and `--project-dir` flags matching your project settings.

# vibemania - comprehensive overview

## 🎯 what is vibemania?

**vibemania** is a macOS app that manages AI coding agents. It provides a clean, intuitive interface for running multiple AI tools on your coding projects simultaneously, with support for various agent types including a unique **playground mode** for building projects from scratch.

---

## 🏗️ architecture overview

### core components

```
vibemania/
├── Models/
│   ├── Project.swift          # Project data model with tool types
│   ├── Agent.swift             # Agent state and lifecycle
│   ├── ProjectStore.swift      # Project persistence & management
│   └── AgentManager.swift      # Agent execution & lifecycle
│
├── Views/
│   ├── ContentView.swift       # Main navigation structure
│   ├── SidebarView.swift       # Project list sidebar
│   ├── DashboardView.swift     # Overview with Liquid Glass widgets
│   ├── ProjectDetailView.swift # Individual project view
│   ├── AddProjectSheet.swift   # Project creation with playground
│   ├── AgentCardView.swift     # Agent status card
│   └── LogView.swift           # Agent log viewer
│
└── Scripts/
    ├── vibe.sh                 # Vibe Code agent script
    ├── ralph.sh                # Ralph agent script  
    └── playground.sh           # Playground mode script
```

---

## 🚀 how it works

### 1. project management

**Projects** are the foundation of vibemania. Each project represents:
- A directory on your Mac containing code
- A tool type (vibe code, ralph, or playground)
- Configuration settings (max iterations, language, framework, etc.)

**Creating a Project:**
1. Click "+" in sidebar
2. Choose project name and path
3. Select tool type
4. Configure settings (varies by tool type)
5. For playground mode: select language, framework, stack, and description

**Tool Types:**
- **vibe code**: Standard AI coding agent
- **ralph (amp)**: Alternative agent with Anthropic's Model Provider
- **ralph (claude)**: Alternative agent using Claude directly
- **playground**: Creates projects from scratch with language awareness

### 2. agent lifecycle

**Agents** are instances of AI tools running on a project:

```
idle → running → completed/failed/stopped
```

**Launching an Agent:**
1. Select a project
2. Click "launch agent"
3. Agent executes in background
4. Real-time logs stream to UI
5. Status updates automatically

**Agent State Management:**
- **Idle**: Agent created but not started
- **Running**: Actively executing iterations
- **Completed**: Finished all iterations successfully
- **Failed**: Encountered an error
- **Stopped**: Manually stopped by user

### 3. playground mode 🎪

Playground mode is vibemania's unique feature for building projects from scratch using AI.

**How Playground Works:**

1. **Language Detection**
   - Scans your system for installed languages (Swift, Node, Python, Ruby, Go, Rust, etc.)
   - Checks project directory for existing code
   - Presents available languages in dropdown

2. **Framework Selection**
   - Optional framework specification (SwiftUI, React, Django, etc.)
   - Helps AI understand your target stack

3. **Stack Definition**
   - Free-form text to describe tech stack
   - Examples: "react + typescript + tailwind", "swift + swiftui + combine"

4. **Project Description**
   - Describe what you want to build
   - AI uses this as the goal for the project
   - More detail = better results

5. **Iterative Building**
   - playground.sh script initializes project structure
   - Creates appropriate starter files (Package.swift, package.json, etc.)
   - AI builds project incrementally over iterations
   - Checks for completion (can build/run)
   - Continues until complete or max iterations reached

**playground.sh Script Flow:**
```bash
Start
  ↓
Detect Languages (system + project)
  ↓
Initialize Project Structure
  ↓
Loop (up to max iterations):
  │
  ├─ Build Context Prompt
  ├─ AI Makes Changes
  ├─ Check if Complete
  └─ Continue or Exit
  ↓
Generate Summary
  ↓
Done
```

### 4. dashboard & monitoring

**Dashboard View** (with Liquid Glass! ✨):
- **Projects**: Total project count
- **Running**: Number of active agents
- **Total Agents**: All agents ever created
- **Completed**: Successfully finished agents

**Real-time Monitoring:**
- Active agents section shows currently running
- Recent agents section shows latest activity
- Each card displays:
  - Project name & tool type
  - Status & iteration progress
  - Duration & progress bar
  - Quick stop button

### 5. liquid glass design 🌊

Vibemania uses Apple's new **Liquid Glass** design language for a modern, premium macOS feel.

**Where Liquid Glass is Applied:**
- **Dashboard stat cards**: Interactive glass with color tints
- **Agent cards**: Subtle glass effect with status color tints
- **Buttons**: Glass button styles throughout

**Benefits:**
- Blurs background content beautifully
- Reflects surrounding colors and light
- Reacts to user interactions
- Creates depth and hierarchy
- Feels native to macOS

**Implementation:**
```swift
// Simple glass effect
.glassEffect()

// Customized glass with tint and interaction
.glassEffect(.regular.tint(.blue.opacity(0.1)).interactive())

// Multiple glass elements that can merge
GlassEffectContainer(spacing: 30.0) {
    HStack {
        StatCard(...)
        StatCard(...)
    }
}
```

---

## 💻 technical details

### data persistence

**ProjectStore:**
- Manages all projects
- Persists to JSON file in app support directory
- Observable with SwiftUI `@Observable` macro
- Methods: `addProject()`, `removeProject()`, `updateProject()`

**AgentManager:**
- Manages all agents across all projects
- Launches agents as subprocess
- Captures real-time logs
- Tracks status and timing
- Methods: `launchAgent()`, `stopAgent()`, `removeAgent()`

### script execution

Agents run as bash scripts in subprocesses:

```swift
// Simplified agent launch
let process = Process()
process.executableURL = URL(fileURLWithPath: scriptPath)
process.arguments = [projectPath, maxIterations, ...]
process.standardOutput = pipe
process.launch()
```

Scripts output to stdout, which is:
- Captured by vibemania
- Parsed for status updates
- Displayed in LogView
- Stored in agent history

### script interface

All scripts follow this interface:

```bash
script.sh <project_path> <max_iterations> [script_specific_args...]
```

**For playground.sh specifically:**
```bash
playground.sh <project_path> <max_iterations> <language> <framework> <stack> <description>
```

Scripts create `.vibemania/` directory in project for:
- Logs (`playground.log`)
- State tracking (`initialized`, `current_prompt.txt`)
- Summaries (`summary.txt`)

---

## 🎨 design philosophy

### lowercase aesthetic
All UI text uses lowercase for consistent branding:
- "dashboard" not "Dashboard"
- "add project" not "Add Project"
- "launch agent" not "Launch Agent"

### macOS native
- Uses native controls (List, Form, Picker, etc.)
- Follows macOS design patterns
- Keyboard shortcuts throughout
- Context menus on interactive elements

### information density
- Dashboard provides quick overview
- Project detail shows everything about one project
- Agent cards balance detail with glanceability
- Logs available when needed but not overwhelming

---

## 🔄 data flow

```
User Action
    ↓
SwiftUI View
    ↓
Store/Manager (@Observable)
    ↓
Model Update
    ↓
UI Automatically Updates
```

**Example: Launching an Agent**
```
1. User clicks "launch agent" button
2. ProjectDetailView calls agentManager.launchAgent(for: project)
3. AgentManager creates Agent model
4. AgentManager spawns Process with appropriate script
5. Agent status updates to .running
6. SwiftUI observes change and updates UI
7. Logs stream in real-time to LogView
8. Agent completes, status updates to .completed
9. UI updates automatically
```

---

## 🎯 key features

### ✅ implemented

- [x] Multiple project support
- [x] Multiple agent types (vibe, ralph, playground)
- [x] Real-time agent monitoring
- [x] Live log streaming
- [x] Playground mode with language detection
- [x] Framework & stack configuration
- [x] Liquid Glass design throughout
- [x] Lowercase "vibemania" branding
- [x] macOS native design
- [x] Project persistence
- [x] Agent history
- [x] Context menus
- [x] Keyboard shortcuts

### 🚧 potential future features

See `FEATURE_SUGGESTIONS.md` for extensive list including:
- Agent templates & presets
- Multi-agent coordination
- Agent history & rollback with git integration
- Smart notifications
- Performance analytics
- Custom scripts & hooks
- Agent chat interface
- Cloud sync
- iOS companion app
- Team collaboration
- And 30+ more ideas!

---

## 🧪 testing playground mode

**To test playground functionality:**

1. **Create a new playground project:**
   - Click "+" in sidebar
   - Name it "test playground"
   - Choose an empty directory
   - Select "playground" as tool type
   - Click refresh to scan languages
   - Choose a language (e.g., "swift")
   - Framework: "swiftui" (optional)
   - Stack: "swift + swiftui + swift concurrency"
   - Description: "create a simple timer app with start/stop button and countdown display"
   - Set max iterations: 15
   - Click "add project"

2. **Launch the agent:**
   - Select the project in sidebar
   - Click "launch agent"
   - Watch as playground.sh:
     - Detects languages
     - Initializes project structure
     - Creates Package.swift (for Swift)
     - Begins iterative development
     - Checks for completion each iteration
     - Generates summary when done

3. **Monitor progress:**
   - Dashboard shows running agent
   - Project detail shows iteration count
   - Logs stream in real-time
   - Agent completes when project can build

---

## 🔧 development workflow

### adding a new tool type

1. **Add to Project.swift:**
```swift
enum ToolType: String, Codable, CaseIterable, Identifiable {
    case newTool = "my new tool"
    
    var scriptName: String {
        switch self {
        case .newTool: return "newtool.sh"
        // ...
        }
    }
}
```

2. **Create script:**
```bash
#!/bin/bash
# newtool.sh
PROJECT_PATH="$1"
MAX_ITERATIONS="$2"
# Your implementation
```

3. **Update AddProjectSheet:**
Add any tool-specific configuration UI

4. **Update AgentManager:**
Add script path resolution if needed

### customizing liquid glass

**Adjust tint colors:**
```swift
.glassEffect(.regular.tint(.purple.opacity(0.15)).interactive())
```

**Change shapes:**
```swift
.glassEffect(.regular, in: .rect(cornerRadius: 20))
.glassEffect(.regular, in: .capsule)
.glassEffect(.regular, in: .circle)
```

**Adjust container spacing:**
```swift
GlassEffectContainer(spacing: 50.0) { // Larger = merge at greater distance
    // Your views
}
```

---

## 🎓 learning resources

### understanding the codebase

**Start here:**
1. `Project.swift` - Data model
2. `ProjectStore.swift` - How projects are managed
3. `AddProjectSheet.swift` - Project creation flow
4. `DashboardView.swift` - Liquid Glass examples
5. `playground.sh` - Script implementation

**Key concepts:**
- SwiftUI `@Observable` macro for state management
- Swift Concurrency (async/await) for async operations
- Process API for subprocess execution
- FileManager for file system operations
- Combine for real-time updates (if used)

### swiftui patterns used

- **Environment objects**: `@Environment(\.dismiss)`, `@Environment(ProjectStore.self)`
- **State management**: `@State`, `@Binding`
- **Observable pattern**: `@Observable` macro on stores
- **Navigation**: `NavigationSplitView`, `List(selection:)`
- **Sheets & modals**: `.sheet(isPresented:)`
- **Context menus**: `.contextMenu { }`

---

## 🐛 debugging tips

### agent not starting
- Check script permissions: `chmod +x scripts/*.sh`
- Verify script path in project bundle
- Check console for Process errors

### language detection not working
- Verify commands installed: `which swift`, `which node`, etc.
- Check PATH environment variable
- Try manual selection if auto-detect fails

### liquid glass not appearing
- Requires macOS 15+ (or iOS 18+)
- May not work in previews, test on device
- Check for typos in modifier syntax

### logs not streaming
- Verify stdout pipe configuration
- Check for buffering issues
- Ensure script outputs to stdout, not stderr

---

## 📚 additional resources

**Apple Documentation:**
- [SwiftUI Liquid Glass Guide](https://developer.apple.com/documentation/SwiftUI/Applying-Liquid-Glass-to-custom-views)
- [Process Documentation](https://developer.apple.com/documentation/foundation/process)
- [Observable Macro](https://developer.apple.com/documentation/Observation)

**Project Files:**
- `FEATURE_SUGGESTIONS.md` - Future roadmap
- `playground.sh` - Playground implementation
- Individual view files - Implementation examples

---

## 🎉 getting started as a user

1. **Launch vibemania**
2. **Add your first project:**
   - Click "+" in sidebar
   - Browse to your project directory
   - Choose tool type (start with "vibe code" for existing projects)
   - Set max iterations (10 is a good default)
3. **Launch an agent:**
   - Select the project
   - Click "launch agent"
   - Watch the magic happen!
4. **Try playground mode:**
   - Click "+" again
   - Choose "playground" tool type
   - Select a language
   - Describe what you want to build
   - Let AI create it from scratch!

---

## 🎉 getting started as a developer

1. **Understand the architecture** (see above)
2. **Read through key files** in this order:
   - `Project.swift`
   - `ProjectStore.swift`
   - `DashboardView.swift`
   - `AddProjectSheet.swift`
   - `playground.sh`
3. **Make a small change:**
   - Try adding a new stat card to dashboard
   - Adjust Liquid Glass tint colors
   - Add a new language to playground detection
4. **Test thoroughly:**
   - Build and run the app
   - Create test projects
   - Launch agents and monitor behavior
5. **Refer to `FEATURE_SUGGESTIONS.md`** for ideas on what to build next!

---

## 🤝 contributing

Want to improve vibemania? Here's how:

1. **Pick a feature** from `FEATURE_SUGGESTIONS.md` or propose your own
2. **Open an issue** to discuss approach
3. **Fork and implement** following existing patterns
4. **Test thoroughly** on real projects
5. **Submit PR** with description of changes

**Coding standards:**
- Follow existing naming conventions (lowercase UI text!)
- Use SwiftUI best practices
- Add Liquid Glass to new UI elements
- Document complex logic
- Keep functions focused and readable

---

## 📝 notes

- vibemania is designed for **macOS** - uses AppKit APIs via NSOpenPanel
- **Scripts must be executable**: `chmod +x *.sh`
- **Playground mode** works best with max iterations 15-30
- **Liquid Glass** requires modern macOS (Sonoma or later recommended)
- **Language detection** checks both system commands and project files

---

## ✨ conclusion

**vibemania** transforms AI coding agents from command-line tools into a delightful macOS experience. With playground mode, you can build entire projects from scratch. With Liquid Glass, everything looks stunning. And with the lowercase aesthetic, it's distinctly *vibemania*.

Happy coding! 🚀✨

---

*last updated: february 2026*
*version: 1.0.0*
*made with ❤️ and ai*

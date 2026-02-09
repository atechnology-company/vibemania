# vibemania Quick Reference

## Tool Types

| Type | Use Case | Script |
|------|----------|--------|
| **vibe code** | Standard Vibe workflow | vibe.sh |
| **ralph (amp)** | Amp-powered coding | ralph.sh --tool amp |
| **ralph (claude)** | Claude-powered coding | ralph.sh --tool claude |
| **playground** | Build from scratch | playground.sh |

## Playground Mode Fields

| Field | Required | Description |
|-------|----------|-------------|
| **language** | Yes | Auto-detected from project files |
| **framework** | No | e.g., "SwiftUI", "React", "Django" |
| **stack** | No | e.g., "react + typescript + tailwind" |
| **description** | Yes | What you want the agent to build |
| **max iterations** | Yes | How many cycles (default: 10) |

## Detected Languages

- swift
- javascript/typescript
- python
- ruby
- go
- rust
- java/kotlin
- c/c++
- c#
- php

## Language Detection Triggers

| Language | Extensions | Config Files |
|----------|-----------|--------------|
| Swift | .swift | Package.swift |
| JS/TS | .js, .jsx, .ts, .tsx | package.json |
| Python | .py | requirements.txt, setup.py |
| Ruby | .rb | Gemfile, Rakefile |
| Go | .go | go.mod, go.sum |
| Rust | .rs | Cargo.toml |
| Java/Kotlin | .java, .kt | build.gradle, pom.xml |
| C/C++ | .c, .cpp, .h | CMakeLists.txt, Makefile |
| C# | .cs | .csproj, .sln |
| PHP | .php | composer.json |

## Playground Script Arguments

```bash
playground.sh \
  --language "swift" \
  --framework "SwiftUI" \
  --stack "swiftui + swift concurrency" \
  --description "Build a weather app with location services" \
  --iterations 20
```

## Liquid Glass Stat Cards

**New Design:**
- Translucent backgrounds
- Color tints per metric
- Interactive hover effects
- Smooth animations

**Old vs New:**
```
Old: .background(.background)
New: .glassEffect(.regular.tint(color.opacity(0.1)).interactive())
```

## Project Model Changes

**New Fields:**
```swift
var playgroundLanguage: String?
var playgroundFramework: String?
var playgroundStack: String?
var playgroundDescription: String?
```

## Keyboard Shortcuts (Future)

| Shortcut | Action |
|----------|--------|
| ⌘N | Add project |
| ⌘R | Launch agent |
| ⌘. | Stop agent |
| ⌘1-9 | Switch projects |
| ⌘F | Search |
| ⌘O | Open in Finder |

## Agent Status Types

- **idle**: Not yet started
- **running**: Currently executing
- **completed**: Finished successfully
- **failed**: Encountered an error
- **stopped**: Manually terminated

## File Locations

**Projects Data:**
```
~/Library/Application Support/VibeMania/projects.json
```

**Playground Script:**
```
/path/to/project/playground.sh
```

## Common Tasks

### Add Standard Project
1. Click "add project"
2. Enter name and path
3. Select tool type (vibe/ralph)
4. Set max iterations
5. Add

### Add Playground Project
1. Click "add project"
2. Enter name and path
3. Select "playground"
4. Wait for language scan (or click refresh)
5. Select language
6. Fill framework/stack (optional)
7. Write project description
8. Add

### Launch Agent
1. Select project in sidebar
2. Click "launch agent" button
3. Monitor logs in real-time
4. Stop if needed

### Remove Project
1. Right-click project in sidebar
2. Select "remove"
3. Confirms deletion (and stops agents)

## Tips

✓ Use descriptive project names
✓ Set realistic max iterations
✓ Write clear playground descriptions
✓ Review logs regularly
✓ Use dashboard for overview
✓ Try different tool types
✓ Experiment with playground mode

## Troubleshooting

**No languages detected:**
- Ensure project has recognizable files
- Click refresh button
- Manually select "unknown"

**Script not found:**
- Check script exists: `ls playground.sh`
- Make executable: `chmod +x playground.sh`
- Verify path is correct

**Agent won't start:**
- Check logs for errors
- Verify script permissions
- Ensure path exists
- Check tool dependencies

## Resources

- `IMPLEMENTATION_SUMMARY.md` - Complete architecture
- `PLAYGROUND_README.md` - Playground guide
- `FEATURE_SUGGESTIONS.md` - Future features
- `VISUAL_GUIDE.md` - Visual changes
- `playground.sh` - Script template

## Support

Need help? Check the documentation files or:
- Open an issue on GitHub
- Review logs in the UI
- Check script output
- Verify file permissions

---

**vibemania** - ai coding agent manager ✨

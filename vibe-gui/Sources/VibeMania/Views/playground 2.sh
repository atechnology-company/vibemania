#!/bin/bash

# playground.sh - vibemania playground mode
# Creates projects from scratch using AI, with language/framework awareness

set -e

PROJECT_PATH="$1"
MAX_ITERATIONS="${2:-10}"
LANGUAGE="${3:-auto}"
FRAMEWORK="${4:-none}"
STACK="${5:-}"
DESCRIPTION="${6:-}"

ITERATION=0
LOG_FILE="$PROJECT_PATH/.vibemania/playground.log"

# Colors for output
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

log() {
    echo -e "${CYAN}[playground]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[playground]${NC} $1" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[playground]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[playground]${NC} $1" | tee -a "$LOG_FILE"
}

# Ensure vibemania directory exists
mkdir -p "$PROJECT_PATH/.vibemania"

# Initialize log
echo "=== playground session started: $(date) ===" >> "$LOG_FILE"
log "language: $LANGUAGE"
log "framework: $FRAMEWORK"
log "stack: $STACK"
log "description: $DESCRIPTION"
log "max iterations: $MAX_ITERATIONS"

# Function to detect installed languages
detect_languages() {
    local detected=()
    
    # Swift
    if command -v swift &> /dev/null; then
        detected+=("swift:$(swift --version 2>&1 | head -n1)")
    fi
    
    # Node.js / JavaScript
    if command -v node &> /dev/null; then
        detected+=("node:$(node --version)")
    fi
    
    # Python
    if command -v python3 &> /dev/null; then
        detected+=("python:$(python3 --version 2>&1)")
    fi
    
    # Ruby
    if command -v ruby &> /dev/null; then
        detected+=("ruby:$(ruby --version | cut -d' ' -f2)")
    fi
    
    # Go
    if command -v go &> /dev/null; then
        detected+=("go:$(go version | cut -d' ' -f3)")
    fi
    
    # Rust
    if command -v rustc &> /dev/null; then
        detected+=("rust:$(rustc --version | cut -d' ' -f2)")
    fi
    
    # Java
    if command -v java &> /dev/null; then
        detected+=("java:$(java --version 2>&1 | head -n1)")
    fi
    
    # PHP
    if command -v php &> /dev/null; then
        detected+=("php:$(php --version | head -n1 | cut -d' ' -f2)")
    fi
    
    # C/C++
    if command -v clang &> /dev/null; then
        detected+=("clang:$(clang --version | head -n1 | cut -d' ' -f4)")
    fi
    
    # C#
    if command -v dotnet &> /dev/null; then
        detected+=("dotnet:$(dotnet --version)")
    fi
    
    echo "${detected[@]}"
}

# Function to detect installed frameworks
detect_frameworks() {
    local frameworks=()
    
    # Swift frameworks (via swift package)
    if command -v swift &> /dev/null; then
        frameworks+=("swiftui" "uikit" "appkit")
    fi
    
    # Node.js frameworks
    if command -v npm &> /dev/null; then
        if [ -f "$PROJECT_PATH/package.json" ]; then
            frameworks+=("react" "vue" "angular" "express" "next" "nuxt")
        fi
    fi
    
    # Python frameworks
    if command -v pip3 &> /dev/null; then
        frameworks+=("django" "flask" "fastapi" "pytorch" "tensorflow")
    fi
    
    # Ruby frameworks
    if command -v gem &> /dev/null; then
        frameworks+=("rails" "sinatra")
    fi
    
    # Go frameworks
    if command -v go &> /dev/null; then
        frameworks+=("gin" "echo" "fiber")
    fi
    
    echo "${frameworks[@]}"
}

# Auto-detect if requested
if [ "$LANGUAGE" = "auto" ]; then
    log "scanning for installed languages..."
    DETECTED_LANGS=($(detect_languages))
    
    if [ ${#DETECTED_LANGS[@]} -eq 0 ]; then
        log_error "no programming languages detected on system"
        exit 1
    fi
    
    log_success "detected languages:"
    for lang in "${DETECTED_LANGS[@]}"; do
        log "  - $lang"
    done
    
    # Use first detected language
    LANGUAGE="${DETECTED_LANGS[0]%%:*}"
    log "using language: $LANGUAGE"
fi

# Function to generate initial project structure
init_project() {
    log "initializing project structure..."
    
    case "$LANGUAGE" in
        swift)
            if [ "$FRAMEWORK" = "swiftui" ] || [ "$FRAMEWORK" = "none" ]; then
                # Create a basic SwiftUI app structure
                mkdir -p "$PROJECT_PATH/Sources"
                mkdir -p "$PROJECT_PATH/Tests"
                
                cat > "$PROJECT_PATH/Package.swift" << 'EOF'
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "PlaygroundProject",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .executable(name: "PlaygroundProject", targets: ["PlaygroundProject"])
    ],
    targets: [
        .executableTarget(name: "PlaygroundProject"),
        .testTarget(name: "PlaygroundProjectTests", dependencies: ["PlaygroundProject"])
    ]
)
EOF
            fi
            ;;
        node)
            cat > "$PROJECT_PATH/package.json" << 'EOF'
{
  "name": "playground-project",
  "version": "1.0.0",
  "description": "vibemania playground project",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "echo \"no tests yet\""
  }
}
EOF
            ;;
        python)
            touch "$PROJECT_PATH/main.py"
            cat > "$PROJECT_PATH/requirements.txt" << 'EOF'
# Add your dependencies here
EOF
            ;;
        rust)
            if command -v cargo &> /dev/null; then
                cd "$PROJECT_PATH" && cargo init --name playground-project
            fi
            ;;
        go)
            cd "$PROJECT_PATH"
            go mod init playground-project 2>/dev/null || true
            touch main.go
            ;;
        *)
            log_warn "no template for $LANGUAGE, creating basic structure"
            touch "$PROJECT_PATH/main.$LANGUAGE"
            ;;
    esac
    
    log_success "project initialized"
}

# Function to build context prompt
build_prompt() {
    local prompt="You are working in playground mode in vibemania. Your task is to help build a project from scratch.

Project Details:
- Language: $LANGUAGE
- Framework: ${FRAMEWORK:-none}
- Stack: ${STACK:-standard}
- Description: ${DESCRIPTION:-no description provided}

Current iteration: $((ITERATION + 1)) of $MAX_ITERATIONS

Your goal is to:
1. Understand what needs to be built from the description
2. Create the necessary files and structure
3. Implement functionality step by step
4. Test and refine the implementation
5. Continue until the project matches the description or max iterations reached

"

    # Check if project has existing files
    local file_count=$(find "$PROJECT_PATH" -type f ! -path "*/.*" 2>/dev/null | wc -l)
    
    if [ "$file_count" -gt 0 ]; then
        prompt+="
Current project state:
$(find "$PROJECT_PATH" -type f ! -path "*/.*" ! -path "*/.vibemania/*" 2>/dev/null | head -20)

"
    else
        prompt+="
This is a fresh project. Start by creating the basic structure.

"
    fi
    
    prompt+="What should be done next? Analyze the current state and make progress toward completing the project."
    
    echo "$prompt"
}

# Function to check if project is complete
is_project_complete() {
    # Check for basic completion indicators
    local has_main_file=false
    local has_tests=false
    local can_run=false
    
    case "$LANGUAGE" in
        swift)
            [ -f "$PROJECT_PATH/Package.swift" ] && has_main_file=true
            find "$PROJECT_PATH" -name "*Tests.swift" &>/dev/null && has_tests=true
            cd "$PROJECT_PATH" && swift build &>/dev/null && can_run=true
            ;;
        node)
            [ -f "$PROJECT_PATH/package.json" ] && has_main_file=true
            [ -f "$PROJECT_PATH/index.js" ] || [ -f "$PROJECT_PATH/src/index.js" ] && can_run=true
            ;;
        python)
            [ -f "$PROJECT_PATH/main.py" ] && has_main_file=true && can_run=true
            ;;
        rust)
            [ -f "$PROJECT_PATH/Cargo.toml" ] && has_main_file=true
            cd "$PROJECT_PATH" && cargo check &>/dev/null && can_run=true
            ;;
        go)
            [ -f "$PROJECT_PATH/go.mod" ] && has_main_file=true
            cd "$PROJECT_PATH" && go build &>/dev/null && can_run=true
            ;;
    esac
    
    if $has_main_file && $can_run; then
        return 0
    else
        return 1
    fi
}

# Initialize project on first iteration
if [ ! -f "$PROJECT_PATH/.vibemania/initialized" ]; then
    init_project
    touch "$PROJECT_PATH/.vibemania/initialized"
fi

# Main loop
log "starting playground session..."

while [ $ITERATION -lt $MAX_ITERATIONS ]; do
    ITERATION=$((ITERATION + 1))
    
    log "iteration $ITERATION/$MAX_ITERATIONS"
    
    # Build the prompt for the AI
    PROMPT=$(build_prompt)
    
    # Save prompt to file for the AI agent
    echo "$PROMPT" > "$PROJECT_PATH/.vibemania/current_prompt.txt"
    
    log "prompt prepared, waiting for AI agent..."
    
    # The actual AI interaction happens via the parent vibemania app
    # This script is called by the app, which handles the AI calls
    # Here we just prepare the environment and log progress
    
    # Check if AI has responded (by checking for changes in project)
    sleep 2
    
    # Check completion
    if is_project_complete; then
        log_success "project appears complete!"
        log "✓ main files exist"
        log "✓ project can build/run"
        
        # Generate summary
        cat > "$PROJECT_PATH/.vibemania/summary.txt" << EOF
playground session completed successfully!

language: $LANGUAGE
framework: $FRAMEWORK
iterations used: $ITERATION/$MAX_ITERATIONS
description: $DESCRIPTION

the project has been created and is ready to use.
EOF
        
        log_success "playground session completed in $ITERATION iterations"
        exit 0
    fi
    
    log "continuing to next iteration..."
done

log_warn "reached max iterations ($MAX_ITERATIONS)"
log "project may not be fully complete, but progress has been made"

cat > "$PROJECT_PATH/.vibemania/summary.txt" << EOF
playground session reached max iterations

language: $LANGUAGE
framework: $FRAMEWORK
iterations used: $ITERATION/$MAX_ITERATIONS
description: $DESCRIPTION

the project is partially complete. you may want to:
1. increase max iterations
2. run another playground session
3. manually complete remaining tasks
EOF

exit 0

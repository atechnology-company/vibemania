#!/bin/bash

# playground.sh - AI Agent Playground Script for vibemania
# Place this script in your project root directory
# Make it executable: chmod +x playground.sh

set -e  # Exit on error

# Default values
LANGUAGE=""
FRAMEWORK=""
STACK=""
DESCRIPTION=""
ITERATIONS=10
PROJECT_DIR=$(pwd)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --language)
            LANGUAGE="$2"
            shift 2
            ;;
        --framework)
            FRAMEWORK="$2"
            shift 2
            ;;
        --stack)
            STACK="$2"
            shift 2
            ;;
        --description)
            DESCRIPTION="$2"
            shift 2
            ;;
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            shift
            ;;
    esac
done

# Print header
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     vibemania playground mode          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Display configuration
echo -e "${YELLOW}Configuration:${NC}"
echo "  Language:    $LANGUAGE"
[ -n "$FRAMEWORK" ] && echo "  Framework:   $FRAMEWORK"
[ -n "$STACK" ] && echo "  Stack:       $STACK"
echo "  Iterations:  $ITERATIONS"
echo "  Directory:   $PROJECT_DIR"
echo ""

echo -e "${YELLOW}Project Goal:${NC}"
echo "$DESCRIPTION"
echo ""

# Function to setup project based on language
setup_project() {
    echo -e "${GREEN}→ Setting up project structure...${NC}"
    
    case "$LANGUAGE" in
        swift)
            if [ ! -f "Package.swift" ]; then
                echo "  Creating Swift package..."
                swift package init --type executable
            fi
            ;;
        javascript/typescript)
            if [ ! -f "package.json" ]; then
                echo "  Creating package.json..."
                npm init -y
            fi
            if [[ "$STACK" == *"typescript"* ]] && [ ! -f "tsconfig.json" ]; then
                echo "  Setting up TypeScript..."
                npm install --save-dev typescript
                npx tsc --init
            fi
            ;;
        python)
            if [ ! -f "requirements.txt" ]; then
                echo "  Creating requirements.txt..."
                touch requirements.txt
            fi
            if [ ! -d "venv" ]; then
                echo "  Creating virtual environment..."
                python3 -m venv venv
            fi
            ;;
        go)
            if [ ! -f "go.mod" ]; then
                echo "  Initializing Go module..."
                go mod init $(basename "$PROJECT_DIR")
            fi
            ;;
        rust)
            if [ ! -f "Cargo.toml" ]; then
                echo "  Creating Cargo project..."
                cargo init
            fi
            ;;
        ruby)
            if [ ! -f "Gemfile" ]; then
                echo "  Creating Gemfile..."
                echo "source 'https://rubygems.org'" > Gemfile
            fi
            ;;
    esac
    
    echo -e "${GREEN}✓ Project structure ready${NC}"
    echo ""
}

# Function to run an iteration
run_iteration() {
    local iteration=$1
    
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Iteration $iteration of $ITERATIONS"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
    
    # TODO: This is where you integrate with your AI coding tool
    # Examples:
    
    # Using Aider (if installed)
    if command -v aider &> /dev/null; then
        echo -e "${GREEN}→ Running Aider...${NC}"
        # aider --message "$DESCRIPTION" --yes
    fi
    
    # Using Cursor CLI (if available)
    # cursor --message "$DESCRIPTION"
    
    # Using custom AI agent
    # python ai_agent.py --prompt "$DESCRIPTION" --iteration $iteration
    
    # Placeholder: Simulate work
    echo "  Working on project..."
    sleep 2
    
    # Check if work is complete (implement your own logic)
    # return 0 if complete, 1 if needs more iterations
    
    echo ""
}

# Function to validate completion
check_completion() {
    echo -e "${YELLOW}→ Checking project completion...${NC}"
    
    # TODO: Implement project-specific completion checks
    # Examples:
    
    case "$LANGUAGE" in
        swift)
            if swift build &> /dev/null; then
                echo -e "${GREEN}✓ Swift build successful${NC}"
                return 0
            fi
            ;;
        javascript/typescript)
            if npm test &> /dev/null; then
                echo -e "${GREEN}✓ Tests passing${NC}"
                return 0
            fi
            ;;
        python)
            if [ -f "venv/bin/activate" ]; then
                source venv/bin/activate
                if python -m pytest &> /dev/null; then
                    echo -e "${GREEN}✓ Tests passing${NC}"
                    deactivate
                    return 0
                fi
                deactivate
            fi
            ;;
        go)
            if go test ./... &> /dev/null; then
                echo -e "${GREEN}✓ Tests passing${NC}"
                return 0
            fi
            ;;
        rust)
            if cargo test &> /dev/null; then
                echo -e "${GREEN}✓ Tests passing${NC}"
                return 0
            fi
            ;;
    esac
    
    return 1
}

# Main execution
main() {
    # Setup project
    setup_project
    
    # Run iterations
    for i in $(seq 1 $ITERATIONS); do
        run_iteration $i
        
        # Check if project is complete
        if check_completion; then
            echo ""
            echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
            echo -e "${GREEN}║  Project completed successfully!       ║${NC}"
            echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
            exit 0
        fi
        
        # Give a moment between iterations
        [ $i -lt $ITERATIONS ] && sleep 1
    done
    
    # Max iterations reached
    echo ""
    echo -e "${YELLOW}╔════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  Max iterations reached                ║${NC}"
    echo -e "${YELLOW}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Project may be incomplete. Consider:${NC}"
    echo "  • Increasing max iterations"
    echo "  • Refining project description"
    echo "  • Running another session"
}

# Run main function
main

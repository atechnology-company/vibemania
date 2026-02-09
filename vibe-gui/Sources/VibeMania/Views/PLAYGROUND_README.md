# Playground Mode Guide

## Overview

Playground mode in vibemania allows you to create experimental projects that use AI agents to build software from scratch based on your description. The agent will continue working until the project is completed.

## How It Works

1. **Create a Playground Project**
   - Click "add project" in the sidebar
   - Select "playground" as the tool type
   - Choose a directory for your new project

2. **Configure Your Project**
   - **Language Detection**: vibemania automatically scans your project directory to detect installed languages and frameworks
   - **Language**: Select from detected languages (Swift, JavaScript/TypeScript, Python, Ruby, Go, Rust, Java/Kotlin, C/C++, C#, PHP)
   - **Framework** (optional): Specify a framework like "SwiftUI", "React", "Django", etc.
   - **Stack**: Define your technology stack (e.g., "react + typescript + tailwind")
   - **Description**: Describe what you want the agent to build

3. **The Agent Works**
   - The playground agent reads your description
   - It sets up the project structure
   - Writes code iteratively
   - Tests and refines until completion
   - Continues working until max iterations or successful completion

## Playground Script Template

Create a `playground.sh` script in your project directory:

```bash
#!/bin/bash

# playground.sh - AI Agent Playground Script
# This script is called by vibemania to run playground mode

LANGUAGE=""
FRAMEWORK=""
STACK=""
DESCRIPTION=""
ITERATIONS=10

# Parse arguments
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
            shift
            ;;
    esac
done

echo "=== Playground Mode ==="
echo "Language: $LANGUAGE"
echo "Framework: $FRAMEWORK"
echo "Stack: $STACK"
echo "Max Iterations: $ITERATIONS"
echo ""
echo "Project Description:"
echo "$DESCRIPTION"
echo ""
echo "========================"
echo ""

# TODO: Integrate with your AI coding tool here
# This is where you'd call your AI agent with the configuration
# For example, you might call a custom script that uses Claude or GPT-4

# Example structure:
# for i in $(seq 1 $ITERATIONS); do
#     echo "Iteration $i of $ITERATIONS"
#     # Call your AI agent
#     # Check if project is complete
#     # Continue or break
# done

echo "Playground mode completed!"
```

## Language Detection

vibemania detects languages by scanning your project for:

- **File Extensions**: `.swift`, `.js`, `.py`, `.rb`, `.go`, `.rs`, etc.
- **Configuration Files**: `Package.swift`, `package.json`, `requirements.txt`, `Cargo.toml`, etc.
- **Project Structure**: Framework-specific directories and files

## Use Cases

- **Rapid Prototyping**: Build MVPs quickly with AI assistance
- **Learning**: Explore new languages and frameworks
- **Experimentation**: Try out ideas without manual setup
- **Code Generation**: Generate boilerplate and scaffolding automatically

## Best Practices

1. **Clear Descriptions**: Provide detailed, clear descriptions of what you want to build
2. **Appropriate Iterations**: Set realistic max iterations (10-50 depending on complexity)
3. **Stack Definition**: Be specific about your technology choices
4. **Directory Organization**: Use clean, organized project directories
5. **Version Control**: Initialize git repositories for your playgrounds

## Advanced Configuration

### Custom AI Integration

Modify the `playground.sh` script to integrate with your preferred AI coding tool:

- **Vibe Code**: For terminal-based coding
- **Aider**: For GPT-4 assisted development
- **Cursor**: For IDE-integrated AI
- **Custom Solutions**: Your own AI agent implementations

### Framework-Specific Setup

Add framework-specific initialization in your playground script:

```bash
if [ "$FRAMEWORK" = "SwiftUI" ]; then
    # Initialize SwiftUI project
    swift package init --type executable
elif [ "$FRAMEWORK" = "React" ]; then
    # Initialize React project
    npx create-react-app .
fi
```

## Troubleshooting

- **No Languages Detected**: Ensure your project directory contains recognizable files
- **Script Not Found**: Verify `playground.sh` exists and is executable (`chmod +x playground.sh`)
- **Agent Doesn't Start**: Check the logs in vibemania for error messages
- **Incorrect Language**: Manually select the correct language from the dropdown

## Future Features

Planned enhancements for playground mode:

- **Template Library**: Pre-built project templates
- **Auto-Install Dependencies**: Automatic package manager setup
- **Success Detection**: Automatic detection of project completion
- **Code Quality Checks**: Built-in linting and testing
- **Multi-Language Projects**: Support for polyglot projects
- **Cloud Integration**: Deploy playgrounds directly to cloud platforms

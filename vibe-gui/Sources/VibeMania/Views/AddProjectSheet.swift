import SwiftUI

struct AddProjectSheet: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(\.dismiss) private var dismiss

    @State private var name = ""
    @State private var path = ""
    @State private var toolType: Project.ToolType = .vibe
    @State private var maxIterations = 10
    
    // Playground-specific fields
    @State private var playgroundLanguage = ""
    @State private var playgroundFramework = ""
    @State private var playgroundStack = ""
    @State private var playgroundDescription = ""
    @State private var availableLanguages: [String] = []
    @State private var isScanning = false

    private var isValid: Bool {
        let baseValid = !name.trimmingCharacters(in: .whitespaces).isEmpty
            && !path.trimmingCharacters(in: .whitespaces).isEmpty
        
        if toolType == .playground {
            return baseValid 
                && !playgroundLanguage.isEmpty 
                && !playgroundDescription.isEmpty
        }
        return baseValid
    }

    var body: some View {
        VStack(spacing: 0) {
            Text("add project")
                .font(.title2)
                .fontWeight(.semibold)
                .padding()

            Form {
                TextField("project name", text: $name)

                HStack {
                    TextField("project path", text: $path)
                    Button("browse…") {
                        let panel = NSOpenPanel()
                        panel.canChooseDirectories = true
                        panel.canChooseFiles = false
                        panel.allowsMultipleSelection = false
                        panel.message = "select the project directory"

                        if panel.runModal() == .OK, let url = panel.url {
                            path = url.path
                            if name.isEmpty {
                                name = url.lastPathComponent
                            }
                            // Auto-scan for languages if playground mode
                            if toolType == .playground {
                                scanForLanguages()
                            }
                        }
                    }
                }

                Picker("tool", selection: $toolType) {
                    ForEach(Project.ToolType.allCases) { type in
                        HStack {
                            Image(systemName: type.icon)
                            Text(type.rawValue)
                        }.tag(type)
                    }
                }
                .onChange(of: toolType) { _, newValue in
                    if newValue == .playground && !path.isEmpty {
                        scanForLanguages()
                    }
                }
                
                if toolType == .playground {
                    Divider()
                    
                    Text("playground configuration")
                        .font(.headline)
                        .padding(.top, 8)
                    
                    HStack {
                        Picker("language", selection: $playgroundLanguage) {
                            Text("select language").tag("")
                            ForEach(availableLanguages, id: \.self) { lang in
                                Text(lang).tag(lang)
                            }
                        }
                        
                        if isScanning {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Button {
                                scanForLanguages()
                            } label: {
                                Image(systemName: "arrow.clockwise")
                            }
                            .buttonStyle(.plain)
                            .disabled(path.isEmpty)
                        }
                    }
                    
                    TextField("framework (optional)", text: $playgroundFramework)
                    
                    TextField("stack (e.g., react + typescript)", text: $playgroundStack)
                    
                    VStack(alignment: .leading) {
                        Text("project description")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        TextEditor(text: $playgroundDescription)
                            .frame(height: 80)
                            .font(.body)
                            .overlay {
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(Color.secondary.opacity(0.2), lineWidth: 1)
                            }
                    }
                } else {
                    Stepper("max iterations: \(maxIterations)", value: $maxIterations, in: 1...100)
                }
            }
            .formStyle(.grouped)
            .padding()

            HStack {
                Button("cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("add project") {
                    let project = Project(
                        name: name.trimmingCharacters(in: .whitespaces),
                        path: path.trimmingCharacters(in: .whitespaces),
                        toolType: toolType,
                        maxIterations: maxIterations,
                        playgroundLanguage: toolType == .playground ? playgroundLanguage : nil,
                        playgroundFramework: toolType == .playground && !playgroundFramework.isEmpty ? playgroundFramework : nil,
                        playgroundStack: toolType == .playground && !playgroundStack.isEmpty ? playgroundStack : nil,
                        playgroundDescription: toolType == .playground ? playgroundDescription : nil
                    )
                    projectStore.addProject(project)
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                .disabled(!isValid)
            }
            .padding()
        }
        .frame(width: 550, height: toolType == .playground ? 600 : 400)
    }
    
    private func scanForLanguages() {
        guard !path.isEmpty else { return }
        
        isScanning = true
        
        Task {
            let languages = await detectLanguages(at: path)
            await MainActor.run {
                availableLanguages = languages
                if !languages.isEmpty && playgroundLanguage.isEmpty {
                    playgroundLanguage = languages[0]
                }
                isScanning = false
            }
        }
    }
    
    private func detectLanguages(at projectPath: String) async -> [String] {
        var detectedLanguages: Set<String> = []
        let fileManager = FileManager.default
        
        // Check system-installed languages first
        var systemLanguages: [String] = []
        
        // Check for installed language runtimes
        let languageCommands = [
            ("swift", "swift"),
            ("javascript/typescript", "node"),
            ("python", "python3"),
            ("ruby", "ruby"),
            ("go", "go"),
            ("rust", "rustc"),
            ("java/kotlin", "java"),
            ("c/c++", "clang"),
            ("c#", "dotnet"),
            ("php", "php")
        ]
        
        for (language, command) in languageCommands {
            if await checkCommand(command) {
                systemLanguages.append(language)
            }
        }
        
        // If directory exists, scan for project files
        if fileManager.fileExists(atPath: projectPath) {
            // Common language indicators
            let languagePatterns: [(extensions: [String], frameworks: [String], language: String)] = [
                (["swift"], ["Package.swift"], "swift"),
                (["js", "jsx", "ts", "tsx"], ["package.json", "node_modules"], "javascript/typescript"),
                (["py"], ["requirements.txt", "setup.py", "pyproject.toml"], "python"),
                (["rb"], ["Gemfile", "Rakefile"], "ruby"),
                (["go"], ["go.mod", "go.sum"], "go"),
                (["rs"], ["Cargo.toml"], "rust"),
                (["java", "kt", "kts"], ["build.gradle", "pom.xml"], "java/kotlin"),
                (["c", "cpp", "cc", "h", "hpp"], ["CMakeLists.txt", "Makefile"], "c/c++"),
                (["cs"], [".csproj", ".sln"], "c#"),
                (["php"], ["composer.json"], "php"),
            ]
            
            guard let enumerator = fileManager.enumerator(atPath: projectPath) else {
                return systemLanguages.isEmpty ? ["unknown"] : systemLanguages
            }
            
            var fileCount = 0
            for case let file as String in enumerator {
                fileCount += 1
                if fileCount > 1000 { break } // Limit scanning for performance
                
                // Skip hidden files and common directories
                if file.hasPrefix(".") || file.contains("node_modules") 
                    || file.contains(".git") || file.contains("build") {
                    continue
                }
                
                for pattern in languagePatterns {
                    // Check file extensions
                    if let ext = file.split(separator: ".").last.map(String.init),
                       pattern.extensions.contains(ext) {
                        detectedLanguages.insert(pattern.language)
                    }
                    
                    // Check for framework files
                    for framework in pattern.frameworks {
                        if file.hasSuffix(framework) {
                            detectedLanguages.insert(pattern.language)
                        }
                    }
                }
            }
        }
        
        // Combine system and project-detected languages, prioritizing project files
        let projectLanguages = Array(detectedLanguages).sorted()
        if !projectLanguages.isEmpty {
            return projectLanguages
        }
        
        return systemLanguages.isEmpty ? ["unknown"] : systemLanguages
    }
    
    private func checkCommand(_ command: String) async -> Bool {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/which")
        process.arguments = [command]
        process.standardOutput = Pipe()
        process.standardError = Pipe()
        
        do {
            try process.run()
            process.waitUntilExit()
            return process.terminationStatus == 0
        } catch {
            return false
        }
    }
}

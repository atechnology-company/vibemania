import SwiftUI

struct AddProjectSheet: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(\.dismiss) private var dismiss

    @State private var name = ""
    @State private var path = ""
    @State private var toolType: Project.ToolType = .claude
    @State private var maxIterations = 10

    private var isValid: Bool {
        !name.trimmingCharacters(in: .whitespaces).isEmpty
            && !path.trimmingCharacters(in: .whitespaces).isEmpty
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
                    Button("browse...") {
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

                Stepper("max iterations: \(maxIterations)", value: $maxIterations, in: 1...100)

                Section {
                    Text(
                        "VibeMania will auto-detect your project's tech stack and use a two-phase AI loop: a planner decides what to do, then an executor implements it."
                    )
                    .font(.caption)
                    .foregroundStyle(.secondary)
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
                        maxIterations: maxIterations
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
        .frame(width: 500, height: 400)
    }
}

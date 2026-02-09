import SwiftUI

struct AddProjectSheet: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(\.dismiss) private var dismiss

    @State private var name = ""
    @State private var path = ""
    @State private var toolType: Project.ToolType = .vibe
    @State private var maxIterations = 10

    private var isValid: Bool {
        !name.trimmingCharacters(in: .whitespaces).isEmpty
            && !path.trimmingCharacters(in: .whitespaces).isEmpty
    }

    var body: some View {
        VStack(spacing: 0) {
            Text("Add Project")
                .font(.title2)
                .fontWeight(.semibold)
                .padding()

            Form {
                TextField("Project Name", text: $name)

                HStack {
                    TextField("Project Path", text: $path)
                    Button("Browse…") {
                        let panel = NSOpenPanel()
                        panel.canChooseDirectories = true
                        panel.canChooseFiles = false
                        panel.allowsMultipleSelection = false
                        panel.message = "Select the project directory"

                        if panel.runModal() == .OK, let url = panel.url {
                            path = url.path
                            if name.isEmpty {
                                name = url.lastPathComponent
                            }
                        }
                    }
                }

                Picker("Tool", selection: $toolType) {
                    ForEach(Project.ToolType.allCases) { type in
                        Text(type.rawValue).tag(type)
                    }
                }

                Stepper("Max Iterations: \(maxIterations)", value: $maxIterations, in: 1...100)
            }
            .formStyle(.grouped)
            .padding()

            HStack {
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("Add Project") {
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
        .frame(width: 500)
    }
}

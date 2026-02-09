import AppKit
import SwiftUI

struct WelcomeView: View {
    @Environment(ConversationStore.self) private var conversationStore
    @Environment(AgentManager.self) private var agentManager
    @State private var goalText = ""
    @State private var selectedProjectPath: String?
    @Binding var selectedConversationId: UUID?

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            Image(systemName: "wand.and.stars")
                .font(.system(size: 64))
                .foregroundStyle(.blue)

            Text("Let's build")
                .font(.largeTitle)
                .fontWeight(.bold)

            Text("VibeMania")
                .font(.title2)
                .foregroundStyle(.secondary)

            Spacer().frame(height: 40)

            VStack(spacing: 16) {
                TextField("Build anything", text: $goalText)
                    .textFieldStyle(.roundedBorder)
                    .font(.title3)
                    .frame(maxWidth: 600)
                    .onSubmit {
                        if !goalText.isEmpty && selectedProjectPath != nil {
                            startConversation()
                        }
                    }

                // Project path selector
                HStack {
                    if let path = selectedProjectPath {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Project:")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Text(path)
                                .font(.callout)
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                        .padding(.horizontal, 12)
                        .padding(.vertical, 8)
                        .frame(maxWidth: 500)
                        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 8))
                    } else {
                        Text("No project selected")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                    }

                    Button {
                        selectProjectPath()
                    } label: {
                        Label("Choose Folder", systemImage: "folder")
                    }
                    .buttonStyle(.bordered)
                }
                .frame(maxWidth: 600)

                HStack(spacing: 16) {
                    Button("Local") {
                        startConversation()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(goalText.isEmpty || selectedProjectPath == nil)

                    Button("Worktree") {
                        startConversation()
                    }
                    .buttonStyle(.bordered)
                    .disabled(goalText.isEmpty || selectedProjectPath == nil)

                    Button("Cloud") {
                        startConversation()
                    }
                    .buttonStyle(.bordered)
                    .disabled(goalText.isEmpty || selectedProjectPath == nil)
                }
            }

            Spacer()

            VStack(spacing: 8) {
                Text("AI-powered development loop")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("Planner AI → Multiple Executor AIs → Conflict Resolution")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            .padding(.bottom, 32)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private func selectProjectPath() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.message = "Select project directory for VibeMania"
        panel.prompt = "Choose"

        if panel.runModal() == .OK {
            selectedProjectPath = panel.url?.path
        }
    }

    private func startConversation() {
        guard !goalText.isEmpty, let projectPath = selectedProjectPath else { return }

        // Create new conversation session
        let session = ConversationSession(
            title: goalText.prefix(50).description, projectPath: projectPath)
        let userMessage = ChatMessage(role: .user, content: goalText)
        session.messages.append(userMessage)

        conversationStore.addSession(session)
        selectedConversationId = session.id

        // Clear input
        goalText = ""

        // Launch agent swarm
        agentManager.launchSwarm(for: session)
    }
}

import AppKit
import SwiftUI

struct ConversationView: View {
    let session: ConversationSession
    @State private var showAgentPanel = true
    @State private var selectedTab = 0
    @State private var selectedAgentId: UUID?
    @Environment(AgentManager.self) private var agentManager

    private var agents: [Agent] {
        session.agents.compactMap { agentId in
            agentManager.agents.first(where: { $0.id == agentId })
        }
    }

    private var selectedAgent: Agent? {
        guard let id = selectedAgentId else { return agents.first }
        return agents.first(where: { $0.id == id })
    }

    private var conflicts: [AgentManager.FileConflict] {
        agentManager.detectConflicts(for: session)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Conflict warning banner
            if !conflicts.isEmpty {
                ConflictWarningBanner(conflicts: conflicts, session: session)
            }

            HSplitView {
                // Left: Chat
                ChatView(session: session)
                    .frame(minWidth: 400)

                // Right: Agent Swarm + Logs
                if showAgentPanel {
                    VStack(spacing: 0) {
                        // Tab selector
                        Picker("View", selection: $selectedTab) {
                            Text("Agents").tag(0)
                            Text("Tasks").tag(1)
                            Text("Logs").tag(2)
                        }
                        .pickerStyle(.segmented)
                        .padding()

                        Divider()

                        // Content
                        if selectedTab == 0 {
                            AgentSwarmView(session: session)
                        } else if selectedTab == 1 {
                            TaskProgressGridView(tasks: session.tasks)
                        } else {
                            if let agent = selectedAgent {
                                VStack(spacing: 0) {
                                    // Agent selector
                                    if agents.count > 1 {
                                        Picker("Agent", selection: $selectedAgentId) {
                                            ForEach(agents) { agent in
                                                Text(
                                                    "\(agent.role.rawValue) - \(agent.status.rawValue)"
                                                )
                                                .tag(agent.id as UUID?)
                                            }
                                        }
                                        .pickerStyle(.menu)
                                        .padding()
                                        Divider()
                                    }

                                    LogView(agent: agent)
                                }
                            } else {
                                ContentUnavailableView(
                                    "no agent selected",
                                    systemImage: "terminal",
                                    description: Text("select an agent to view its logs")
                                )
                            }
                        }
                    }
                    .frame(minWidth: 350, idealWidth: 450)
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    withAnimation {
                        showAgentPanel.toggle()
                    }
                } label: {
                    Image(systemName: showAgentPanel ? "sidebar.right" : "sidebar.left")
                }
                .help(showAgentPanel ? "Hide agent panel" : "Show agent panel")
            }

            ToolbarItem(placement: .automatic) {
                Menu {
                    Button {
                        exportConversation()
                    } label: {
                        Label("Export conversation", systemImage: "square.and.arrow.up")
                    }

                    Button(role: .destructive) {
                        // Stop all agents
                        for agent in agents where agent.isRunning {
                            agentManager.stopAgent(agent)
                        }
                    } label: {
                        Label("Stop all agents", systemImage: "stop.fill")
                    }
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
        }
        .navigationTitle(session.title)
        .onAppear {
            // Set first agent as selected by default
            if selectedAgentId == nil {
                selectedAgentId = agents.first?.id
            }
        }
    }

    // MARK: - Export

    private func exportConversation() {
        let markdown = generateMarkdown()

        let panel = NSSavePanel()
        panel.nameFieldStringValue = "\(session.title.replacingOccurrences(of: " ", with: "-")).md"
        panel.allowedContentTypes = [.plainText]
        panel.message = "Export conversation to Markdown"

        if panel.runModal() == .OK, let url = panel.url {
            do {
                try markdown.write(to: url, atomically: true, encoding: .utf8)
            } catch {
                print("Failed to export conversation: \(error)")
            }
        }
    }

    private func generateMarkdown() -> String {
        var markdown = """
            # \(session.title)

            **Created:** \(session.createdAt.formatted(date: .long, time: .shortened))
            **Project:** \(session.projectPath ?? "N/A")
            **Agents:** \(session.agents.count)
            **Tasks:** \(session.tasks.count)

            ---

            ## Conversation


            """

        // Add messages
        for message in session.messages {
            let roleIcon: String
            switch message.role {
            case .user: roleIcon = "👤"
            case .planner: roleIcon = "🧠"
            case .executor: roleIcon = "🔨"
            case .system: roleIcon = "⚙️"
            }

            markdown += """
                ### \(roleIcon) \(message.role.rawValue.capitalized) - \(message.timestamp.formatted(date: .omitted, time: .shortened))

                \(message.content)


                """
        }

        // Add tasks section
        if !session.tasks.isEmpty {
            markdown += """
                ---

                ## Tasks


                """

            for (index, task) in session.tasks.enumerated() {
                let statusIcon: String
                switch task.status {
                case .pending: statusIcon = "⏳"
                case .running: statusIcon = "▶️"
                case .completed: statusIcon = "✅"
                case .failed: statusIcon = "❌"
                case .blocked: statusIcon = "🚫"
                }

                markdown += """
                    ### \(statusIcon) Task \(index + 1): \(task.title)

                    **Status:** \(task.status.rawValue)
                    **Files Affected:** \(task.filesAffected.isEmpty ? "None" : task.filesAffected.joined(separator: ", "))

                    \(task.description)


                    """
            }
        }

        // Add conflicts section
        let conflicts = agentManager.detectConflicts(for: session)
        if !conflicts.isEmpty {
            markdown += """
                ---

                ## ⚠️ Conflicts Detected


                """

            for conflict in conflicts {
                markdown += """
                    ### \(conflict.file)

                    Modified by \(conflict.agentIds.count) agents


                    """
            }
        }

        markdown += """
            ---

            *Exported from VibeMania*
            """

        return markdown
    }
}

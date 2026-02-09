import SwiftUI

struct ContentView: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(AgentManager.self) private var agentManager
    @Environment(ConversationStore.self) private var conversationStore

    @State private var mode: ViewMode = .conversations  // New default: Codex-style
    @State private var selectedProjectId: UUID?
    @State private var selectedConversationId: UUID?
    @State private var showingAddProject = false
    @State private var showingDashboard = false

    enum ViewMode: String {
        case conversations
        case projects
    }

    var selectedConversation: ConversationSession? {
        guard let id = selectedConversationId else { return nil }
        return conversationStore.sessions.first(where: { $0.id == id })
    }

    var body: some View {
        NavigationSplitView {
            VStack(spacing: 0) {
                // Large oval mode toggle at top - macOS 26 style
                ModeSelectorView(mode: $mode)
                    .padding(.top, 20)
                    .padding(.horizontal, 16)
                    .padding(.bottom, 16)

                Divider()

                // Sidebar content based on mode
                if mode == .conversations {
                    ConversationListView(selectedId: $selectedConversationId)
                } else {
                    SidebarView(
                        selectedProjectId: $selectedProjectId,
                        showingDashboard: $showingDashboard,
                        showingAddProject: $showingAddProject
                    )
                }
            }
        } detail: {
            if mode == .conversations {
                if let session = selectedConversation {
                    ConversationView(session: session)
                } else {
                    WelcomeView(selectedConversationId: $selectedConversationId)
                }
            } else {
                if showingDashboard {
                    DashboardView()
                } else if let projectId = selectedProjectId,
                    let project = projectStore.projects.first(where: { $0.id == projectId })
                {
                    ProjectDetailView(project: project)
                } else {
                    ContentUnavailableView(
                        "select a project",
                        systemImage: "folder",
                        description: Text("choose a project from the sidebar or add a new one.")
                    )
                }
            }
        }
        .sheet(isPresented: $showingAddProject) {
            AddProjectSheet()
        }
        .frame(minWidth: 1200, minHeight: 700)
    }

    // MARK: - Command Actions

    private func createNewConversation() {
        mode = .conversations
        selectedConversationId = nil
        showingDashboard = false
    }

    private func exportCurrentConversation() {
        // This would need access to the current ConversationView
        // For now, just a placeholder
        print("Export conversation action triggered")
    }

    private func stopAllAgents() {
        for agent in agentManager.runningAgents {
            agentManager.stopAgent(agent)
        }
    }
}

// MARK: - Mode Selector View (macOS 26 Liquid Glass Style)

struct ModeSelectorView: View {
    @Binding var mode: ContentView.ViewMode
    @Namespace private var animation

    var body: some View {
        HStack(spacing: 0) {
            ModeButton(
                title: "conversations",
                icon: "message.fill",
                isSelected: mode == .conversations,
                namespace: animation
            ) {
                withAnimation(.spring(response: 0.3, dampingFraction: 0.7)) {
                    mode = .conversations
                }
            }

            ModeButton(
                title: "projects",
                icon: "folder.fill",
                isSelected: mode == .projects,
                namespace: animation
            ) {
                withAnimation(.spring(response: 0.3, dampingFraction: 0.7)) {
                    mode = .projects
                }
            }
        }
        .padding(4)
        .background(.ultraThinMaterial, in: Capsule())
        .overlay(
            Capsule()
                .strokeBorder(.quaternary, lineWidth: 0.5)
        )
    }
}

struct ModeButton: View {
    let title: String
    let icon: String
    let isSelected: Bool
    let namespace: Namespace.ID
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                Image(systemName: icon)
                    .font(.system(size: 14, weight: .medium))
                Text(title)
                    .font(.system(size: 14, weight: .medium))
            }
            .foregroundStyle(isSelected ? .primary : .secondary)
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .frame(maxWidth: .infinity)
            .background {
                if isSelected {
                    Capsule()
                        .fill(.background)
                        .shadow(color: .black.opacity(0.1), radius: 2, y: 1)
                        .matchedGeometryEffect(id: "mode_selector", in: namespace)
                }
            }
        }
        .buttonStyle(.plain)
    }
}

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
                // Mode toggle
                Picker("Mode", selection: $mode) {
                    Label("Conversations", systemImage: "message.fill").tag(ViewMode.conversations)
                    Label("Projects", systemImage: "folder.fill").tag(ViewMode.projects)
                }
                .pickerStyle(.segmented)
                .padding()

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
}

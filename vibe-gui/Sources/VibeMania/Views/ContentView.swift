import SwiftUI

struct ContentView: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(AgentManager.self) private var agentManager
    @State private var selectedProjectId: UUID?
    @State private var showingAddProject = false
    @State private var showingDashboard = true

    var body: some View {
        NavigationSplitView {
            SidebarView(
                selectedProjectId: $selectedProjectId,
                showingDashboard: $showingDashboard,
                showingAddProject: $showingAddProject
            )
        } detail: {
            if showingDashboard {
                DashboardView()
            } else if let projectId = selectedProjectId,
                      let project = projectStore.projects.first(where: { $0.id == projectId }) {
                ProjectDetailView(project: project)
            } else {
                ContentUnavailableView(
                    "select a project",
                    systemImage: "folder",
                    description: Text("choose a project from the sidebar or add a new one.")
                )
            }
        }
        .sheet(isPresented: $showingAddProject) {
            AddProjectSheet()
        }
        .frame(minWidth: 900, minHeight: 600)
    }
}

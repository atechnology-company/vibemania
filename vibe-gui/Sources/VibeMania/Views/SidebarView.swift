import SwiftUI

struct SidebarView: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(AgentManager.self) private var agentManager
    @Binding var selectedProjectId: UUID?
    @Binding var showingDashboard: Bool
    @Binding var showingAddProject: Bool
    
    @State private var isProjectsSectionHovered = false

    var body: some View {
        List(selection: $selectedProjectId) {
            Section {
                Button {
                    showingDashboard = true
                    selectedProjectId = nil
                } label: {
                    Label("dashboard", systemImage: "square.grid.2x2")
                }
                .buttonStyle(.plain)
                .padding(.vertical, 4)
                .foregroundStyle(showingDashboard ? Color.accentColor : .primary)
            }

            Section {
                HStack {
                    Text("projects")
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundStyle(.secondary)
                        .textCase(.uppercase)
                    
                    Spacer()
                    
                    Button {
                        showingAddProject = true
                    } label: {
                        Image(systemName: "plus.circle.fill")
                            .font(.system(size: 16))
                            .foregroundStyle(.secondary)
                            .symbolRenderingMode(.hierarchical)
                    }
                    .buttonStyle(.plain)
                    .opacity(isProjectsSectionHovered ? 1.0 : 0.0)
                    .animation(.easeInOut(duration: 0.2), value: isProjectsSectionHovered)
                    .help("Add Project")
                }
                .padding(.vertical, 4)
                .onHover { hovering in
                    isProjectsSectionHovered = hovering
                }
                
                ForEach(projectStore.projects) { project in
                    let runningCount = agentManager
                        .agentsForProject(project.id)
                        .filter(\.isRunning)
                        .count

                    ProjectRow(project: project, runningAgentCount: runningCount)
                        .tag(project.id)
                        .onTapGesture {
                            selectedProjectId = project.id
                            showingDashboard = false
                        }
                        .contextMenu {
                            Button("remove", role: .destructive) {
                                for agent in agentManager.agentsForProject(project.id) {
                                    agentManager.removeAgent(agent)
                                }
                                projectStore.removeProject(project)
                                if selectedProjectId == project.id {
                                    selectedProjectId = nil
                                    showingDashboard = true
                                }
                            }
                        }
                }
            }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(min: 220, ideal: 260)
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    showingAddProject = true
                } label: {
                    Label("add project", systemImage: "plus")
                }
            }
        }
    }
}

// MARK: - Project Row

struct ProjectRow: View {
    let project: Project
    let runningAgentCount: Int

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(project.name)
                    .font(.headline)
                Text(project.toolType.rawValue)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            if runningAgentCount > 0 {
                HStack(spacing: 4) {
                    Circle()
                        .fill(.green)
                        .frame(width: 8, height: 8)
                    Text("\(runningAgentCount)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(.vertical, 2)
    }
}

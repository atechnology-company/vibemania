import SwiftUI

struct ProjectDetailView: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(AgentManager.self) private var agentManager
    let project: Project
    @State private var selectedAgentId: UUID?
    @State private var showingSettings = false

    private var agents: [Agent] {
        agentManager.agentsForProject(project.id)
    }

    var body: some View {
        HSplitView {
            // Left column — project info + agent list
            VStack(spacing: 0) {
                projectHeader
                Divider()
                agentList
            }
            .frame(minWidth: 300, idealWidth: 350)

            // Right column — selected agent logs
            if let agentId = selectedAgentId,
               let agent = agents.first(where: { $0.id == agentId }) {
                LogView(agent: agent)
            } else {
                ContentUnavailableView(
                    "Select an Agent",
                    systemImage: "text.alignleft",
                    description: Text("Select an agent to view its logs.")
                )
            }
        }
        .sheet(isPresented: $showingSettings) {
            ProjectSettingsSheet(project: project)
        }
    }

    // MARK: - Subviews

    private var projectHeader: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading) {
                    Text(project.name)
                        .font(.title)
                        .fontWeight(.bold)
                    Text(project.path)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }

                Spacer()

                Button {
                    showingSettings = true
                } label: {
                    Image(systemName: "gear")
                }
            }

            HStack(spacing: 12) {
                Label(project.toolType.rawValue, systemImage: project.toolType.icon)
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(.blue.opacity(0.1), in: Capsule())

                Label("\(project.maxIterations) max iterations", systemImage: "arrow.counterclockwise")
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(.purple.opacity(0.1), in: Capsule())
            }

            HStack {
                Button {
                    let agent = agentManager.launchAgent(for: project)
                    selectedAgentId = agent.id
                } label: {
                    Label("Launch Agent", systemImage: "play.fill")
                }
                .buttonStyle(.borderedProminent)
                .tint(.green)

                if agents.contains(where: \.isRunning) {
                    Button {
                        for agent in agents where agent.isRunning {
                            agentManager.stopAgent(agent)
                        }
                    } label: {
                        Label("Stop All", systemImage: "stop.fill")
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(.red)
                }
            }
        }
        .padding()
    }

    @ViewBuilder
    private var agentList: some View {
        if agents.isEmpty {
            ContentUnavailableView(
                "No Agents",
                systemImage: "bolt.slash",
                description: Text("Launch an agent to start vibe coding.")
            )
        } else {
            List(selection: $selectedAgentId) {
                ForEach(agents.reversed()) { agent in
                    AgentListRow(agent: agent)
                        .tag(agent.id)
                        .contextMenu {
                            if agent.isRunning {
                                Button("Stop") {
                                    agentManager.stopAgent(agent)
                                }
                            }
                            Button("Remove", role: .destructive) {
                                agentManager.removeAgent(agent)
                                if selectedAgentId == agent.id {
                                    selectedAgentId = nil
                                }
                            }
                        }
                }
            }
        }
    }
}

// MARK: - Agent List Row

struct AgentListRow: View {
    let agent: Agent

    private var statusColor: Color {
        switch agent.status {
        case .running:   return .green
        case .completed: return .blue
        case .failed:    return .red
        case .stopped:   return .orange
        case .idle:      return .gray
        }
    }

    var body: some View {
        HStack(spacing: 12) {
            Circle()
                .fill(statusColor)
                .frame(width: 10, height: 10)

            VStack(alignment: .leading, spacing: 2) {
                Text(agent.toolType.rawValue)
                    .font(.headline)
                HStack {
                    Text(agent.status.rawValue)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    if agent.isRunning {
                        Text("• Iteration \(agent.iteration)/\(agent.maxIterations)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Spacer()

            Text(agent.formattedDuration)
                .font(.caption)
                .monospacedDigit()
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Settings Sheet

struct ProjectSettingsSheet: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(\.dismiss) private var dismiss
    let project: Project

    @State private var name: String = ""
    @State private var path: String = ""
    @State private var toolType: Project.ToolType = .vibe
    @State private var maxIterations: Int = 10

    var body: some View {
        VStack(spacing: 0) {
            Text("Project Settings")
                .font(.title2)
                .fontWeight(.semibold)
                .padding()

            Form {
                TextField("Name", text: $name)
                TextField("Path", text: $path)
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
                Button("Save") {
                    var updated = project
                    updated.name = name
                    updated.path = path
                    updated.toolType = toolType
                    updated.maxIterations = maxIterations
                    projectStore.updateProject(updated)
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
            }
            .padding()
        }
        .frame(width: 450)
        .onAppear {
            name = project.name
            path = project.path
            toolType = project.toolType
            maxIterations = project.maxIterations
        }
    }
}

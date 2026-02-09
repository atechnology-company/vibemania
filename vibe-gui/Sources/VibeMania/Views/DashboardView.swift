import SwiftUI

struct DashboardView: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(AgentManager.self) private var agentManager

    var body: some View {
        ScrollView {
            VStack(spacing: 24) {
                // Header
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("vibemania")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                        Text("ai coding agent manager")
                            .font(.title3)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                .padding(.horizontal)

                // Stats with Liquid Glass
                HStack(spacing: 16) {
                    StatCard(
                        title: "projects",
                        value: "\(projectStore.projects.count)",
                        icon: "folder.fill",
                        color: .blue
                    )
                    StatCard(
                        title: "running",
                        value: "\(agentManager.runningAgents.count)",
                        icon: "bolt.fill",
                        color: .green
                    )
                    StatCard(
                        title: "total agents",
                        value: "\(agentManager.agents.count)",
                        icon: "cpu",
                        color: .purple
                    )
                    StatCard(
                        title: "completed",
                        value: "\(agentManager.agents.filter { $0.status == .completed }.count)",
                        icon: "checkmark.circle.fill",
                        color: .orange
                    )
                }
                .padding(.horizontal)

                // Active agents
                if !agentManager.runningAgents.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("active agents")
                            .font(.title2)
                            .fontWeight(.semibold)
                            .padding(.horizontal)

                        ForEach(agentManager.runningAgents) { agent in
                            AgentCardView(agent: agent)
                                .padding(.horizontal)
                        }
                    }
                }

                // Recent agents
                let recentAgents = Array(
                    agentManager.agents
                        .filter { !$0.isRunning }
                        .suffix(5)
                        .reversed()
                )
                if !recentAgents.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("recent agents")
                            .font(.title2)
                            .fontWeight(.semibold)
                            .padding(.horizontal)

                        ForEach(recentAgents) { agent in
                            AgentCardView(agent: agent)
                                .padding(.horizontal)
                        }
                    }
                }

                Spacer()
            }
            .padding(.vertical)
        }
        .background(Color(nsColor: .windowBackgroundColor))
    }
}

// MARK: - Stat Card with Liquid Glass

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color
    
    @State private var isHovering = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Image(systemName: icon)
                .foregroundStyle(color)
                .font(.title2)

            Text(value)
                .font(.system(size: 32, weight: .bold, design: .rounded))

            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(.ultraThinMaterial)
                .overlay {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .fill(color.opacity(isHovering ? 0.15 : 0.08))
                }
                .overlay {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .strokeBorder(.white.opacity(isHovering ? 0.3 : 0.2), lineWidth: 1)
                }
                .shadow(color: .black.opacity(isHovering ? 0.15 : 0.1), radius: isHovering ? 10 : 6, y: isHovering ? 4 : 2)
        }
        .scaleEffect(isHovering ? 1.02 : 1.0)
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isHovering)
        .onHover { hovering in
            isHovering = hovering
        }
    }
}

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
                        Text("VibeMania")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                        Text("Vibe Code Agent Manager")
                            .font(.title3)
                            .foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                .padding(.horizontal)

                // Stats
                HStack(spacing: 16) {
                    StatCard(
                        title: "Projects",
                        value: "\(projectStore.projects.count)",
                        icon: "folder.fill",
                        color: .blue
                    )
                    StatCard(
                        title: "Running",
                        value: "\(agentManager.runningAgents.count)",
                        icon: "bolt.fill",
                        color: .green
                    )
                    StatCard(
                        title: "Total Agents",
                        value: "\(agentManager.agents.count)",
                        icon: "cpu",
                        color: .purple
                    )
                    StatCard(
                        title: "Completed",
                        value: "\(agentManager.agents.filter { $0.status == .completed }.count)",
                        icon: "checkmark.circle.fill",
                        color: .orange
                    )
                }
                .padding(.horizontal)

                // Active agents
                if !agentManager.runningAgents.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Active Agents")
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
                        Text("Recent Agents")
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

// MARK: - Stat Card

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

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
        .background(.background, in: RoundedRectangle(cornerRadius: 12))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color.secondary.opacity(0.2), lineWidth: 1)
        )
    }
}

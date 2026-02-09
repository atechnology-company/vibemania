import SwiftUI

struct AgentSwarmView: View {
    let session: ConversationSession
    @Environment(AgentManager.self) private var agentManager

    var agents: [Agent] {
        session.agents.compactMap { agentId in
            agentManager.agents.first(where: { $0.id == agentId })
        }
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                if agents.isEmpty {
                    ContentUnavailableView(
                        "no agents running",
                        systemImage: "bolt.slash",
                        description: Text("agents will appear here when they start working")
                    )
                } else {
                    ForEach(agents) { agent in
                        AgentSwarmCard(agent: agent, session: session)
                    }
                }
            }
            .padding()
        }
    }
}

struct AgentSwarmCard: View {
    let agent: Agent
    let session: ConversationSession
    @Environment(AgentManager.self) private var agentManager
    @State private var isHovering = false

    private var statusColor: Color {
        switch agent.status {
        case .running: return .green
        case .completed: return .blue
        case .failed: return .red
        case .stopped: return .orange
        case .idle: return .gray
        }
    }

    private var task: AgentTask? {
        guard let taskId = agent.taskId else { return nil }
        return session.tasks.first(where: { $0.id == taskId })
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Header
            HStack {
                Circle()
                    .fill(statusColor)
                    .frame(width: 10, height: 10)

                Text(agent.role.rawValue.capitalized)
                    .font(.headline)

                Spacer()

                if agent.isRunning {
                    Text("Iteration \(agent.iteration)/\(agent.maxIterations)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Text(agent.formattedDuration)
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundStyle(.secondary)

                if agent.isRunning {
                    Button {
                        agentManager.stopAgent(agent)
                    } label: {
                        Image(systemName: "stop.circle.fill")
                            .foregroundStyle(.red)
                    }
                    .buttonStyle(.borderless)
                }
            }

            // Task info
            if let task {
                Text(task.title)
                    .font(.subheadline)
                    .fontWeight(.medium)

                if agent.isRunning {
                    ProgressView(value: Double(agent.iteration), total: Double(agent.maxIterations))
                        .tint(statusColor)
                }

                if !task.filesAffected.isEmpty {
                    HStack {
                        Image(systemName: "doc.text")
                            .font(.caption2)
                        Text(task.filesAffected.prefix(3).joined(separator: ", "))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                        if task.filesAffected.count > 3 {
                            Text("+\(task.filesAffected.count - 3) more")
                                .font(.caption2)
                                .foregroundStyle(.tertiary)
                        }
                    }
                }
            }

            // Recent logs preview
            if !agent.logs.isEmpty {
                let recentLines = agent.logs.split(separator: "\n").suffix(3).joined(
                    separator: "\n")
                if !recentLines.isEmpty {
                    Text(recentLines)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .lineLimit(3)
                        .padding(8)
                        .background(
                            Color(nsColor: .textBackgroundColor),
                            in: RoundedRectangle(cornerRadius: 6))
                }
            }
        }
        .padding()
        .background {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(.ultraThinMaterial)
                .overlay {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .strokeBorder(statusColor.opacity(isHovering ? 0.5 : 0.2), lineWidth: 1.5)
                }
                .shadow(
                    color: .black.opacity(isHovering ? 0.15 : 0.1), radius: isHovering ? 8 : 4,
                    y: isHovering ? 3 : 2)
        }
        .scaleEffect(isHovering ? 1.02 : 1.0)
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isHovering)
        .onHover { hovering in
            isHovering = hovering
        }
    }
}

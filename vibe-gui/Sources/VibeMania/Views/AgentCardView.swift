import SwiftUI

struct AgentCardView: View {
    @Environment(AgentManager.self) private var agentManager
    let agent: Agent
    
    @State private var isHovering = false

    private var statusColor: Color {
        switch agent.status {
        case .running:   return .green
        case .completed: return .blue
        case .failed:    return .red
        case .stopped:   return .orange
        case .idle:      return .gray
        }
    }

    private var statusIcon: String {
        switch agent.status {
        case .running:   return "bolt.fill"
        case .completed: return "checkmark.circle.fill"
        case .failed:    return "xmark.circle.fill"
        case .stopped:   return "stop.circle.fill"
        case .idle:      return "circle"
        }
    }

    var body: some View {
        HStack(spacing: 16) {
            Image(systemName: statusIcon)
                .font(.title2)
                .foregroundStyle(statusColor)
                .frame(width: 32)

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(agent.projectName)
                        .font(.headline)
                    Text("•")
                        .foregroundStyle(.secondary)
                    Text(agent.toolType.rawValue)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                HStack(spacing: 12) {
                    Label(agent.status.rawValue, systemImage: statusIcon)
                        .font(.caption)
                        .foregroundStyle(statusColor)

                    if agent.isRunning {
                        Text("iteration \(agent.iteration)/\(agent.maxIterations)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Text(agent.formattedDuration)
                        .font(.caption)
                        .monospacedDigit()
                        .foregroundStyle(.secondary)
                }

                if agent.isRunning {
                    ProgressView(value: Double(agent.iteration), total: Double(agent.maxIterations))
                        .tint(statusColor)
                }
            }

            Spacer()

            if agent.isRunning {
                Button {
                    agentManager.stopAgent(agent)
                } label: {
                    Image(systemName: "stop.fill")
                        .foregroundStyle(.red)
                }
                .buttonStyle(.borderless)
            }
        }
        .padding()
        .background {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(.ultraThinMaterial)
                .overlay {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .fill(statusColor.opacity(isHovering ? 0.1 : 0.05))
                }
                .overlay {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .strokeBorder(.white.opacity(isHovering ? 0.3 : 0.2), lineWidth: 1)
                }
                .shadow(color: .black.opacity(isHovering ? 0.15 : 0.1), radius: isHovering ? 10 : 6, y: isHovering ? 4 : 2)
        }
        .scaleEffect(isHovering ? 1.01 : 1.0)
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isHovering)
        .onHover { hovering in
            isHovering = hovering
        }
    }
}

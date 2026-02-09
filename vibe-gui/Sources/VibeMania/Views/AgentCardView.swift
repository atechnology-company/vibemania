import SwiftUI

struct AgentCardView: View {
    @Environment(AgentManager.self) private var agentManager
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
                        Text("Iteration \(agent.iteration)/\(agent.maxIterations)")
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
        .background(.background, in: RoundedRectangle(cornerRadius: 12))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(statusColor.opacity(0.3), lineWidth: 1)
        )
    }
}

import SwiftUI

/// Visual progress view for individual tasks
struct TaskProgressView: View {
    let task: AgentTask
    @Environment(AgentManager.self) private var agentManager

    private var assignedAgent: Agent? {
        guard let agentId = task.assignedAgentId else { return nil }
        return agentManager.agents.first(where: { $0.id == agentId })
    }

    private var progress: Double {
        guard let agent = assignedAgent, agent.maxIterations > 0 else { return 0 }
        return Double(agent.iteration) / Double(agent.maxIterations)
    }

    private var statusColor: Color {
        switch task.status {
        case .pending: return .gray
        case .running: return .blue
        case .completed: return .green
        case .failed: return .red
        case .blocked: return .orange
        }
    }

    private var statusIcon: String {
        switch task.status {
        case .pending: return "clock"
        case .running: return "gearshape.2.fill"
        case .completed: return "checkmark.circle.fill"
        case .failed: return "xmark.circle.fill"
        case .blocked: return "exclamationmark.triangle.fill"
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Header with status
            HStack {
                Image(systemName: statusIcon)
                    .foregroundStyle(statusColor)
                    .font(.title3)

                VStack(alignment: .leading, spacing: 2) {
                    Text(task.title)
                        .font(.headline)
                        .lineLimit(1)

                    HStack(spacing: 4) {
                        Text(task.status.rawValue)
                            .font(.caption)
                            .foregroundStyle(.secondary)

                        if let agent = assignedAgent, task.status == .running {
                            Text("•")
                                .font(.caption2)
                                .foregroundStyle(.tertiary)
                            Text("Iteration \(agent.iteration)/\(agent.maxIterations)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                Spacer()

                if let agent = assignedAgent {
                    Text(agent.formattedDuration)
                        .font(.caption)
                        .monospacedDigit()
                        .foregroundStyle(.secondary)
                }
            }

            // Progress bar (only for running tasks)
            if task.status == .running {
                ProgressView(value: progress)
                    .tint(statusColor)
            }

            // Files affected
            if !task.filesAffected.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Files (\(task.filesAffected.count)):")
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    ForEach(task.filesAffected.prefix(3), id: \.self) { file in
                        HStack(spacing: 4) {
                            Image(systemName: "doc.text")
                                .font(.caption2)
                            Text(file)
                                .font(.caption)
                                .lineLimit(1)
                        }
                        .foregroundStyle(.secondary)
                    }

                    if task.filesAffected.count > 3 {
                        Text("+ \(task.filesAffected.count - 3) more")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                }
            }

            // Timestamps
            HStack {
                if let started = task.startedAt {
                    Label(
                        started.formatted(date: .omitted, time: .shortened),
                        systemImage: "play.fill"
                    )
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                }

                if let completed = task.completedAt {
                    Label(
                        completed.formatted(date: .omitted, time: .shortened),
                        systemImage: "checkmark"
                    )
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                }
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(.ultraThinMaterial)
                .overlay(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .strokeBorder(statusColor.opacity(0.3), lineWidth: 1)
                )
        )
    }
}

/// Grid view of all tasks with progress
struct TaskProgressGridView: View {
    let tasks: [AgentTask]

    var body: some View {
        ScrollView {
            LazyVGrid(
                columns: [
                    GridItem(.flexible(), spacing: 16),
                    GridItem(.flexible(), spacing: 16),
                ], spacing: 16
            ) {
                ForEach(tasks) { task in
                    TaskProgressView(task: task)
                }
            }
            .padding()
        }
    }
}

/// Compact task progress bar for sidebar
struct TaskProgressIndicator: View {
    let tasks: [AgentTask]

    private var completedCount: Int {
        tasks.filter { $0.status == .completed }.count
    }

    private var progress: Double {
        guard !tasks.isEmpty else { return 0 }
        return Double(completedCount) / Double(tasks.count)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("\(completedCount)/\(tasks.count) tasks")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Spacer()

                Text("\(Int(progress * 100))%")
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundStyle(.secondary)
            }

            ProgressView(value: progress)
                .tint(.green)
        }
    }
}

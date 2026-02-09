import Foundation

enum AgentStatus: String {
    case idle = "Idle"
    case running = "Running"
    case completed = "Completed"
    case failed = "Failed"
    case stopped = "Stopped"
}

@Observable
final class Agent: Identifiable {
    let id: UUID
    let projectId: UUID
    let projectName: String
    let toolType: Project.ToolType
    var status: AgentStatus = .idle
    var logs: String = ""
    var iteration: Int = 0
    var maxIterations: Int
    var startedAt: Date?
    var endedAt: Date?

    var process: Process?
    var outputPipe: Pipe?

    init(
        id: UUID = UUID(),
        projectId: UUID,
        projectName: String,
        toolType: Project.ToolType,
        maxIterations: Int
    ) {
        self.id = id
        self.projectId = projectId
        self.projectName = projectName
        self.toolType = toolType
        self.maxIterations = maxIterations
    }

    var isRunning: Bool {
        status == .running
    }

    var duration: TimeInterval? {
        guard let start = startedAt else { return nil }
        let end = endedAt ?? Date()
        return end.timeIntervalSince(start)
    }

    var formattedDuration: String {
        guard let duration else { return "--" }
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%02d:%02d", minutes, seconds)
    }
}

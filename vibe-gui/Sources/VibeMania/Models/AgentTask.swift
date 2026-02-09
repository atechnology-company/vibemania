import Foundation

enum TaskStatus: String, Codable {
    case pending
    case running
    case completed
    case failed
    case blocked
}

struct AgentTask: Identifiable, Codable {
    let id: UUID
    let title: String
    let description: String
    let planFile: String  // e.g., "plan-1.md"
    var status: TaskStatus
    var assignedAgentId: UUID?
    var filesAffected: [String]  // Track which files this task modifies
    let createdAt: Date
    var startedAt: Date?
    var completedAt: Date?

    init(
        id: UUID = UUID(), title: String, description: String, planFile: String,
        status: TaskStatus = .pending, filesAffected: [String] = []
    ) {
        self.id = id
        self.title = title
        self.description = description
        self.planFile = planFile
        self.status = status
        self.filesAffected = filesAffected
        self.createdAt = Date()
    }
}

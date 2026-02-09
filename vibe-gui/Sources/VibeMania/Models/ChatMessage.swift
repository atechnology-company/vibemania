import Foundation

enum MessageRole: String, Codable {
    case user
    case planner
    case executor
    case system
}

struct ChatMessage: Identifiable, Codable {
    let id: UUID
    let role: MessageRole
    let content: String
    let timestamp: Date
    let agentId: UUID?  // Link to agent if from planner/executor

    init(
        id: UUID = UUID(), role: MessageRole, content: String, timestamp: Date = Date(),
        agentId: UUID? = nil
    ) {
        self.id = id
        self.role = role
        self.content = content
        self.timestamp = timestamp
        self.agentId = agentId
    }
}

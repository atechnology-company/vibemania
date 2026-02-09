import Foundation

@Observable
final class ConversationSession: Identifiable, Codable {
    let id: UUID
    var projectId: UUID?
    var projectPath: String?  // Path to the project directory
    var title: String  // "Build authentication system"
    var messages: [ChatMessage] = []
    var tasks: [AgentTask] = []
    var agents: [UUID] = []  // Agent IDs involved
    let createdAt: Date
    var isActive: Bool = true

    enum CodingKeys: String, CodingKey {
        case id, projectId, projectPath, title, messages, tasks, agents, createdAt, isActive
    }

    init(id: UUID = UUID(), title: String = "New Conversation", projectPath: String? = nil) {
        self.id = id
        self.title = title
        self.projectPath = projectPath
        self.createdAt = Date()
    }

    required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        projectId = try container.decodeIfPresent(UUID.self, forKey: .projectId)
        projectPath = try container.decodeIfPresent(String.self, forKey: .projectPath)
        title = try container.decode(String.self, forKey: .title)
        messages = try container.decode([ChatMessage].self, forKey: .messages)
        tasks = try container.decode([AgentTask].self, forKey: .tasks)
        agents = try container.decode([UUID].self, forKey: .agents)
        createdAt = try container.decode(Date.self, forKey: .createdAt)
        isActive = try container.decode(Bool.self, forKey: .isActive)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encodeIfPresent(projectId, forKey: .projectId)
        try container.encodeIfPresent(projectPath, forKey: .projectPath)
        try container.encode(title, forKey: .title)
        try container.encode(messages, forKey: .messages)
        try container.encode(tasks, forKey: .tasks)
        try container.encode(agents, forKey: .agents)
        try container.encode(createdAt, forKey: .createdAt)
        try container.encode(isActive, forKey: .isActive)
    }
}

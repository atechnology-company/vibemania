import Foundation

struct Project: Identifiable, Codable, Hashable {
    var id: UUID = UUID()
    var name: String
    var path: String
    var toolType: ToolType = .vibe
    var maxIterations: Int = 10
    var createdAt: Date = Date()
    
    // Playground-specific fields
    var playgroundLanguage: String?
    var playgroundFramework: String?
    var playgroundStack: String?
    var playgroundDescription: String?

    enum ToolType: String, Codable, CaseIterable, Identifiable {
        case vibe = "vibe code"
        case ralph = "ralph (amp)"
        case ralphClaude = "ralph (claude)"
        case playground = "playground"

        var id: String { rawValue }

        var scriptName: String {
            switch self {
            case .vibe: return "vibe.sh"
            case .ralph, .ralphClaude: return "ralph.sh"
            case .playground: return "playground.sh"
            }
        }

        var icon: String {
            switch self {
            case .vibe: return "waveform"
            case .ralph: return "terminal"
            case .ralphClaude: return "terminal.fill"
            case .playground: return "atom"
            }
        }
    }
}

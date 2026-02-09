import Foundation

struct Project: Identifiable, Codable, Hashable {
    var id: UUID = UUID()
    var name: String
    var path: String
    var toolType: ToolType = .vibe
    var maxIterations: Int = 10
    var createdAt: Date = Date()

    enum ToolType: String, Codable, CaseIterable, Identifiable {
        case vibe = "Vibe Code"
        case ralph = "Ralph (Amp)"
        case ralphClaude = "Ralph (Claude)"

        var id: String { rawValue }

        var scriptName: String {
            switch self {
            case .vibe: return "vibe.sh"
            case .ralph, .ralphClaude: return "ralph.sh"
            }
        }

        var icon: String {
            switch self {
            case .vibe: return "waveform"
            case .ralph: return "terminal"
            case .ralphClaude: return "terminal.fill"
            }
        }
    }
}

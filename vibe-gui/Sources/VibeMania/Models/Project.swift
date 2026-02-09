import Foundation

struct Project: Identifiable, Codable, Hashable {
    var id: UUID = UUID()
    var name: String
    var path: String
    var toolType: ToolType = .claude
    var maxIterations: Int = 10
    var createdAt: Date = Date()

    enum ToolType: String, Codable, CaseIterable, Identifiable {
        case claude = "vibemania (claude)"
        case amp = "vibemania (amp)"

        var id: String { rawValue }

        var scriptName: String {
            return "vibemania.sh"
        }

        var toolFlag: String {
            switch self {
            case .claude: return "claude"
            case .amp: return "amp"
            }
        }

        var icon: String {
            switch self {
            case .claude: return "waveform"
            case .amp: return "terminal"
            }
        }
    }
}

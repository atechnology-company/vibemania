import SwiftUI

struct ConflictWarningBanner: View {
    let conflicts: [AgentManager.FileConflict]
    let session: ConversationSession
    @State private var isExpanded = false
    @Environment(AgentManager.self) private var agentManager

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)

                Text("\(conflicts.count) file conflict\(conflicts.count == 1 ? "" : "s") detected")
                    .font(.headline)
                    .foregroundStyle(.primary)

                Spacer()

                Button {
                    withAnimation {
                        isExpanded.toggle()
                    }
                } label: {
                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                }
                .buttonStyle(.borderless)
            }
            .padding()
            .background(.orange.opacity(0.15))

            // Expanded details
            if isExpanded {
                Divider()

                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        ForEach(conflicts) { conflict in
                            ConflictRow(conflict: conflict, agents: agentManager.agents)
                        }
                    }
                    .padding()
                }
                .frame(maxHeight: 200)
                .background(.orange.opacity(0.05))
            }
        }
        .overlay(
            Rectangle()
                .fill(.orange)
                .frame(height: 2),
            alignment: .bottom
        )
    }
}

struct ConflictRow: View {
    let conflict: AgentManager.FileConflict
    let agents: [Agent]

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Image(systemName: "doc.fill")
                    .foregroundStyle(.orange)
                Text(conflict.file)
                    .font(.callout)
                    .fontWeight(.medium)
            }

            Text("Modified by \(conflict.agentIds.count) agents:")
                .font(.caption)
                .foregroundStyle(.secondary)

            ForEach(conflict.agentIds, id: \.self) { agentId in
                if let agent = agents.first(where: { $0.id == agentId }) {
                    HStack(spacing: 6) {
                        Circle()
                            .fill(.orange)
                            .frame(width: 6, height: 6)
                        Text("\(agent.role.rawValue) - \(agent.status.rawValue)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.leading, 8)
                }
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 8))
    }
}

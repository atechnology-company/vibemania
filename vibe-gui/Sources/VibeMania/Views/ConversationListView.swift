import SwiftUI

struct ConversationListView: View {
    @Environment(ConversationStore.self) private var conversationStore
    @Binding var selectedId: UUID?

    var body: some View {
        List(selection: $selectedId) {
            ForEach(conversationStore.sessions.sorted(by: { $0.createdAt > $1.createdAt })) {
                session in
                ConversationListRow(session: session)
                    .tag(session.id)
                    .contextMenu {
                        Button("Remove", role: .destructive) {
                            conversationStore.removeSession(session)
                            if selectedId == session.id {
                                selectedId = nil
                            }
                        }
                    }
            }
        }
    }
}

struct ConversationListRow: View {
    let session: ConversationSession
    @Environment(AgentManager.self) private var agentManager

    private var runningAgentCount: Int {
        session.agents.filter { agentId in
            agentManager.agents.first(where: { $0.id == agentId })?.isRunning == true
        }.count
    }

    var body: some View {
        HStack(spacing: 8) {
            VStack(alignment: .leading, spacing: 4) {
                Text(session.title)
                    .font(.headline)
                    .lineLimit(1)

                HStack(spacing: 4) {
                    if runningAgentCount > 0 {
                        Circle()
                            .fill(.green)
                            .frame(width: 6, height: 6)
                        Text("\(runningAgentCount) running")
                            .font(.caption2)
                            .foregroundStyle(.green)
                    } else {
                        Text("\(session.agents.count) agents")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Text("·")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)

                    Text(session.createdAt, style: .relative)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()

            if runningAgentCount > 0 {
                ProgressView()
                    .scaleEffect(0.7)
                    .controlSize(.small)
            }
        }
        .padding(.vertical, 4)
    }
}

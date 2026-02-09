import SwiftUI

struct ChatView: View {
    let session: ConversationSession
    @State private var inputText = ""
    @Environment(AgentManager.self) private var agentManager

    var body: some View {
        VStack(spacing: 0) {
            messageList
            Divider()
            inputArea
        }
    }

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 12) {
                    ForEach(session.messages) { message in
                        MessageBubble(message: message)
                            .id(message.id)
                    }
                }
                .padding()
                .id("message-list-bottom")
            }
            .onChange(of: session.messages.count) { _, _ in
                withAnimation {
                    proxy.scrollTo("message-list-bottom", anchor: .bottom)
                }
            }
        }
    }

    private var inputArea: some View {
        HStack(spacing: 12) {
            TextField("Build anything", text: $inputText)
                .textFieldStyle(.roundedBorder)
                .font(.body)
                .onSubmit {
                    sendMessage()
                }

            Button {
                sendMessage()
            } label: {
                Image(systemName: "arrow.up.circle.fill")
                    .font(.title2)
            }
            .buttonStyle(.borderless)
            .disabled(inputText.isEmpty)
        }
        .padding()
        .background(.ultraThinMaterial)
    }

    private func sendMessage() {
        guard !inputText.isEmpty else { return }

        let message = ChatMessage(role: .user, content: inputText)
        session.messages.append(message)
        inputText = ""

        // Trigger agent launch if not already running
        if agentManager.agentsForSession(session.id).filter({ $0.isRunning }).isEmpty {
            agentManager.launchSwarm(for: session)
        }
    }
}

struct MessageBubble: View {
    let message: ChatMessage

    private var backgroundColor: Color {
        switch message.role {
        case .user:
            return .blue
        case .planner:
            return .purple.opacity(0.2)
        case .executor:
            return .green.opacity(0.2)
        case .system:
            return .orange.opacity(0.2)
        }
    }

    private var alignment: HorizontalAlignment {
        message.role == .user ? .trailing : .leading
    }

    private var roleIcon: String {
        switch message.role {
        case .user:
            return "person.fill"
        case .planner:
            return "brain"
        case .executor:
            return "hammer.fill"
        case .system:
            return "gear"
        }
    }

    var body: some View {
        HStack {
            if message.role == .user { Spacer(minLength: 60) }

            VStack(alignment: alignment, spacing: 4) {
                // Role label
                if message.role != .user {
                    HStack(spacing: 4) {
                        Image(systemName: roleIcon)
                            .font(.caption2)
                        Text(message.role.rawValue.capitalized)
                            .font(.caption)
                            .fontWeight(.medium)
                    }
                    .foregroundStyle(.secondary)
                }

                // Message content
                Text(message.content)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(backgroundColor, in: RoundedRectangle(cornerRadius: 12))
                    .foregroundColor(message.role == .user ? .white : .primary)

                // Timestamp
                Text(message.timestamp, style: .time)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            .frame(maxWidth: .infinity, alignment: message.role == .user ? .trailing : .leading)

            if message.role != .user { Spacer(minLength: 60) }
        }
    }
}

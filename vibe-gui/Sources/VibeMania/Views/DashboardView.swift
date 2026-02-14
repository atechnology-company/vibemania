import SwiftUI

struct DashboardView: View {
    @Environment(ProjectStore.self) private var projectStore
    @Environment(AgentManager.self) private var agentManager
    
    @State private var inputText = ""
    @State private var selectedModel: ClaudeModel = .sonnet
    @State private var showModelSelector = false
    @State private var attachments: [Attachment] = []

    enum ClaudeModel: String, CaseIterable, Identifiable {
        case haiku = "claude-haiku-4-5"
        case sonnet = "claude-sonnet-4-5"
        case opus = "claude-opus-4-6"

        var id: String { rawValue }

        var displayName: String {
            switch self {
            case .haiku: return "Haiku"
            case .sonnet: return "Sonnet"
            case .opus: return "Opus"
            }
        }

        var subtitle: String {
            switch self {
            case .haiku: return "Fast"
            case .sonnet: return "Balanced"
            case .opus: return "Most Capable"
            }
        }

        var icon: String {
            switch self {
            case .haiku: return "bolt.fill"
            case .sonnet: return "star.fill"
            case .opus: return "crown.fill"
            }
        }

        var color: Color {
            switch self {
            case .haiku: return .cyan
            case .sonnet: return .purple
            case .opus: return .orange
            }
        }
    }
    
    struct Attachment: Identifiable {
        let id = UUID()
        let name: String
        let type: AttachmentType
        
        enum AttachmentType {
            case file
            case folder
            case image
        }
    }

    var body: some View {
        ZStack(alignment: .bottom) {
            // Main content area with bottom padding for input
            ScrollView {
                VStack(spacing: 0) {
                    // Welcome header (empty state)
                    if agentManager.agents.isEmpty {
                        VStack(spacing: 24) {
                            Spacer()

                            Image(systemName: "sparkles")
                                .font(.system(size: 60))
                                .foregroundStyle(.purple.gradient)
                                .symbolEffect(.pulse, options: .repeating)

                            VStack(spacing: 8) {
                                Text("VibeMania")
                                    .font(.system(size: 40, weight: .bold))

                                Text("What can I help you build?")
                                    .font(.title2)
                                    .foregroundStyle(.secondary)
                            }

                            Spacer()
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .padding(.bottom, 240)
                    } else {
                        // Message list
                        LazyVStack(spacing: 24) {
                            ForEach(agentManager.agents) { agent in
                                AgentMessageCard(agent: agent)
                                    .transition(.asymmetric(
                                        insertion: .scale(scale: 0.95).combined(with: .opacity),
                                        removal: .opacity
                                    ))
                            }
                        }
                        .padding(.horizontal, 24)
                        .padding(.top, 24)
                        .padding(.bottom, 240)
                    }
                }
            }
            .background(Color(nsColor: .windowBackgroundColor))

            // Fixed input area at bottom (ChatGPT style)
            VStack(spacing: 0) {
                // Gradient fade at top of input area
                LinearGradient(
                    colors: [
                        Color(nsColor: .windowBackgroundColor).opacity(0),
                        Color(nsColor: .windowBackgroundColor).opacity(0.95),
                        Color(nsColor: .windowBackgroundColor)
                    ],
                    startPoint: .top,
                    endPoint: .bottom
                )
                .frame(height: 60)
                .allowsHitTesting(false)

                inputArea
                    .padding(.horizontal, 20)
                    .padding(.bottom, 24)
                    .padding(.top, 8)
                    .background(Color(nsColor: .windowBackgroundColor))
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: agentManager.agents.isEmpty)
    }
    
    private var inputArea: some View {
        VStack(spacing: 16) {
            // Model Selector - 3-option toggle (ChatGPT/Codex style)
            modelSelectorToggle
                .frame(maxWidth: 700)

            // Main input field
            HStack(alignment: .bottom, spacing: 12) {
                // Attachment button
                Button {
                    // TODO: Add file attachment
                } label: {
                    Image(systemName: "paperclip.circle.fill")
                        .font(.system(size: 24))
                        .foregroundStyle(.secondary)
                        .symbolRenderingMode(.hierarchical)
                }
                .buttonStyle(.plain)
                .help("Attach files")
                .opacity(0.8)
                .scaleEffect(1.0)

                // Text input
                VStack(alignment: .leading, spacing: 8) {
                    // Attachments preview
                    if !attachments.isEmpty {
                        HStack(spacing: 8) {
                            ForEach(attachments) { attachment in
                                AttachmentChip(attachment: attachment) {
                                    withAnimation(.spring(response: 0.3, dampingFraction: 0.7)) {
                                        attachments.removeAll { $0.id == attachment.id }
                                    }
                                }
                            }
                        }
                        .padding(.horizontal, 4)
                        .transition(.asymmetric(
                            insertion: .scale(scale: 0.8).combined(with: .opacity),
                            removal: .scale(scale: 0.8).combined(with: .opacity)
                        ))
                    }

                    // Text field
                    TextField("Message VibeMania...", text: $inputText, axis: .vertical)
                        .textFieldStyle(.plain)
                        .font(.system(size: 15))
                        .lineLimit(1...8)
                        .padding(2)
                        .onSubmit {
                            sendMessage()
                        }
                }
                .frame(maxWidth: .infinity)

                // Send button
                Button {
                    sendMessage()
                } label: {
                    Image(systemName: inputText.isEmpty ? "arrow.up.circle.fill" : "arrow.up.circle.fill")
                        .font(.system(size: 24))
                        .foregroundStyle(inputText.isEmpty ? Color.secondary.opacity(0.5) : selectedModel.color)
                        .symbolRenderingMode(.hierarchical)
                }
                .buttonStyle(.plain)
                .disabled(inputText.isEmpty)
                .help("Send message (⌘↵)")
                .scaleEffect(inputText.isEmpty ? 0.95 : 1.05)
                .animation(.spring(response: 0.3, dampingFraction: 0.6), value: inputText.isEmpty)
            }
            .padding(16)
            .background {
                RoundedRectangle(cornerRadius: 20, style: .continuous)
                    .fill(Color(nsColor: .controlBackgroundColor))
                    .shadow(color: .black.opacity(0.08), radius: 8, x: 0, y: 2)
            }
            .overlay {
                RoundedRectangle(cornerRadius: 20, style: .continuous)
                    .strokeBorder(Color(nsColor: .separatorColor).opacity(0.3), lineWidth: 1)
            }
        }
        .frame(maxWidth: 800)
    }

    private var modelSelectorToggle: some View {
        HStack(spacing: 0) {
            ForEach(ClaudeModel.allCases) { model in
                ModelToggleButton(
                    model: model,
                    isSelected: selectedModel == model
                ) {
                    withAnimation(.spring(response: 0.35, dampingFraction: 0.75)) {
                        selectedModel = model
                    }
                }
            }
        }
        .padding(4)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(.quaternary.opacity(0.5), lineWidth: 1)
        }
        .shadow(color: .black.opacity(0.05), radius: 8, y: 2)
    }
    
    private func sendMessage() {
        guard !inputText.isEmpty else { return }
        
        // TODO: Implement sending message with selected model
        print("Sending message with \(selectedModel.displayName): \(inputText)")
        
        inputText = ""
        attachments.removeAll()
    }
}

// MARK: - Helper Views

struct AttachmentChip: View {
    let attachment: DashboardView.Attachment
    let onRemove: () -> Void
    
    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: iconForType(attachment.type))
                .font(.caption)
                .foregroundStyle(.secondary)
            
            Text(attachment.name)
                .font(.caption)
                .lineLimit(1)
            
            Button {
                onRemove()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(.quaternary.opacity(0.5), in: Capsule())
    }
    
    private func iconForType(_ type: DashboardView.Attachment.AttachmentType) -> String {
        switch type {
        case .file: return "doc.fill"
        case .folder: return "folder.fill"
        case .image: return "photo.fill"
        }
    }
}

struct ModifierButton: View {
    let icon: String
    let label: String
    let isActive: Bool
    let action: () -> Void
    
    @State private var isHovering = false
    
    var body: some View {
        Button {
            action()
        } label: {
            HStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.caption)
                Text(label)
                    .font(.caption)
                    .fontWeight(.medium)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background {
                Capsule()
                    .fill(isActive ? Color.accentColor.opacity(0.2) : Color(nsColor: .controlBackgroundColor))
                    .overlay {
                        if !isActive {
                            Capsule()
                                .strokeBorder(Color(nsColor: .separatorColor).opacity(isHovering ? 0.5 : 0.3), lineWidth: 0.5)
                        }
                    }
            }
            .foregroundStyle(isActive ? Color.accentColor : Color.secondary)
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            isHovering = hovering
        }
    }
}

// MARK: - Model Toggle Button

struct ModelToggleButton: View {
    let model: DashboardView.ClaudeModel
    let isSelected: Bool
    let action: () -> Void

    @State private var isHovering = false

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Image(systemName: model.icon)
                    .font(.system(size: 18, weight: .medium))
                    .foregroundStyle(isSelected ? model.color : .secondary)

                Text(model.displayName)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(isSelected ? .primary : .secondary)

                Text(model.subtitle)
                    .font(.system(size: 10))
                    .foregroundStyle(.tertiary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 10)
            .padding(.horizontal, 12)
            .background {
                if isSelected {
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .fill(Color(nsColor: .controlBackgroundColor))
                        .shadow(color: model.color.opacity(0.2), radius: 4, x: 0, y: 1)
                }
            }
            .scaleEffect(isHovering && !isSelected ? 1.02 : 1.0)
        }
        .buttonStyle(.plain)
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) {
                isHovering = hovering
            }
        }
        .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isSelected)
    }
}

// MARK: - Agent Message Card

struct AgentMessageCard: View {
    let agent: Agent

    @State private var isHovering = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header
            HStack(spacing: 10) {
                Circle()
                    .fill(statusColor)
                    .frame(width: 8, height: 8)
                    .overlay {
                        if agent.status == .running {
                            Circle()
                                .stroke(statusColor.opacity(0.3), lineWidth: 6)
                                .scaleEffect(1.5)
                                .opacity(0.0)
                                .animation(.easeOut(duration: 1.5).repeatForever(autoreverses: false), value: agent.status)
                        }
                    }

                Text(agent.projectName)
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundStyle(.primary)

                Text("•")
                    .foregroundStyle(.tertiary)

                Text(agent.role.rawValue.capitalized)
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)

                Spacer()

                if agent.status == .running {
                    HStack(spacing: 6) {
                        Text(agent.formattedDuration)
                            .font(.system(size: 12, design: .monospaced))
                            .foregroundStyle(.secondary)

                        ProgressView()
                            .controlSize(.small)
                    }
                }
            }

            // Task description
            if let task = agent.task {
                Text(task.description)
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
                    .lineLimit(3)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background {
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .fill(.ultraThinMaterial)
                .overlay {
                    RoundedRectangle(cornerRadius: 14, style: .continuous)
                        .strokeBorder(.quaternary.opacity(0.5), lineWidth: 1)
                }
                .shadow(color: .black.opacity(isHovering ? 0.1 : 0.05), radius: isHovering ? 12 : 6, y: isHovering ? 4 : 2)
        }
        .scaleEffect(isHovering ? 1.005 : 1.0)
        .animation(.spring(response: 0.35, dampingFraction: 0.75), value: isHovering)
        .onHover { hovering in
            isHovering = hovering
        }
    }

    private var statusColor: Color {
        switch agent.status {
        case .running: return .green
        case .completed: return .blue
        case .failed: return .red
        default: return .orange
        }
    }
}
#Preview("Dashboard") {
    DashboardView()
        .environment(ProjectStore())
        .environment(AgentManager())
        .frame(width: 1200, height: 800)
}


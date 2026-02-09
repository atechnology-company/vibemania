import SwiftUI
import AppKit

struct LogView: View {
    let agent: Agent
    @State private var autoScroll = true

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Label(agent.toolType.rawValue, systemImage: "terminal")
                    .font(.headline)

                Spacer()

                Toggle("Auto-scroll", isOn: $autoScroll)
                    .toggleStyle(.switch)
                    .controlSize(.small)

                Button {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(agent.logs, forType: .string)
                } label: {
                    Image(systemName: "doc.on.doc")
                }
                .help("Copy logs to clipboard")
            }
            .padding(.horizontal)
            .padding(.vertical, 8)

            Divider()

            // Log content
            ScrollViewReader { proxy in
                ScrollView {
                    Text(agent.logs.isEmpty ? "Waiting for output..." : agent.logs)
                        .font(.system(.body, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding()
                        .textSelection(.enabled)
                        .id("log-bottom")
                }
                .onChange(of: agent.logs) {
                    if autoScroll {
                        withAnimation {
                            proxy.scrollTo("log-bottom", anchor: .bottom)
                        }
                    }
                }
            }
            .background(Color(nsColor: .textBackgroundColor))
        }
    }
}

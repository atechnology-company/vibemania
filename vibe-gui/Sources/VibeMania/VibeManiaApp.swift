import SwiftUI

@main
struct VibeManiaApp: App {
    @State private var projectStore = ProjectStore()
    @State private var agentManager = AgentManager()
    @State private var conversationStore = ConversationStore()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(projectStore)
                .environment(agentManager)
                .environment(conversationStore)
        }
        .defaultSize(width: 1400, height: 900)
    }
}

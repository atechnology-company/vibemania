import SwiftUI

@main
struct VibeManiaApp: App {
    @State private var projectStore = ProjectStore()
    @State private var agentManager = AgentManager()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(projectStore)
                .environment(agentManager)
        }
        .defaultSize(width: 1200, height: 800)
    }
}

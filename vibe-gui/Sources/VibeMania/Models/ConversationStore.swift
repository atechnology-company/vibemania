import Foundation

@Observable
final class ConversationStore {
    var sessions: [ConversationSession] = []

    private let storageURL: URL

    init() {
        let appSupport = FileManager.default.urls(
            for: .applicationSupportDirectory,
            in: .userDomainMask
        ).first!
        let appDir = appSupport.appendingPathComponent("VibeMania", isDirectory: true)
        try? FileManager.default.createDirectory(at: appDir, withIntermediateDirectories: true)
        storageURL = appDir.appendingPathComponent("conversations.json")
        load()
    }

    func addSession(_ session: ConversationSession) {
        sessions.append(session)
        save()
    }

    func removeSession(_ session: ConversationSession) {
        sessions.removeAll { $0.id == session.id }
        save()
    }

    func updateSession(_ session: ConversationSession) {
        if let index = sessions.firstIndex(where: { $0.id == session.id }) {
            sessions[index] = session
            save()
        }
    }

    // MARK: - Persistence

    private func save() {
        do {
            let data = try JSONEncoder().encode(sessions)
            try data.write(to: storageURL, options: .atomic)
        } catch {
            print("Failed to save conversations: \(error)")
        }
    }

    private func load() {
        guard FileManager.default.fileExists(atPath: storageURL.path) else { return }
        do {
            let data = try Data(contentsOf: storageURL)
            sessions = try JSONDecoder().decode([ConversationSession].self, from: data)
        } catch {
            print("Failed to load conversations: \(error)")
        }
    }
}

import Foundation
import UserNotifications

/// Manages macOS notifications for VibeMania events
final class NotificationManager {

    static let shared = NotificationManager()

    private init() {
        requestAuthorization()
    }

    /// Request notification authorization from the user
    func requestAuthorization() {
        let center = UNUserNotificationCenter.current()
        center.requestAuthorization(options: [.alert, .sound, .badge]) { granted, error in
            if granted {
                print("NotificationManager: Authorization granted")
            } else if let error = error {
                print("NotificationManager: Authorization error: \(error)")
            }
        }
    }

    /// Send notification for task completion
    func notifyTaskCompleted(taskTitle: String, projectName: String) {
        let content = UNMutableNotificationContent()
        content.title = "Task Completed"
        content.body = "\(taskTitle) in \(projectName)"
        content.sound = .default
        content.categoryIdentifier = "TASK_COMPLETED"

        sendNotification(identifier: "task-\(UUID().uuidString)", content: content)
    }

    /// Send notification for task failure
    func notifyTaskFailed(taskTitle: String, projectName: String) {
        let content = UNMutableNotificationContent()
        content.title = "Task Failed"
        content.body = "\(taskTitle) in \(projectName)"
        content.sound = .defaultCritical
        content.categoryIdentifier = "TASK_FAILED"

        sendNotification(identifier: "task-\(UUID().uuidString)", content: content)
    }

    /// Send notification for file conflict
    func notifyConflictDetected(fileName: String, agentCount: Int) {
        let content = UNMutableNotificationContent()
        content.title = "File Conflict Detected"
        content.body = "\(fileName) modified by \(agentCount) agents"
        content.sound = .default
        content.categoryIdentifier = "CONFLICT"

        sendNotification(identifier: "conflict-\(UUID().uuidString)", content: content)
    }

    /// Send notification for all goals completed
    func notifyAllGoalsCompleted(projectName: String) {
        let content = UNMutableNotificationContent()
        content.title = "🎉 All Goals Completed!"
        content.body = "\(projectName) is ready"
        content.sound = .default
        content.categoryIdentifier = "ALL_COMPLETE"

        sendNotification(identifier: "complete-\(UUID().uuidString)", content: content)
    }

    /// Send notification for agent swarm started
    func notifySwarmStarted(agentCount: Int, projectName: String) {
        let content = UNMutableNotificationContent()
        content.title = "Agent Swarm Started"
        content.body = "\(agentCount) agents working on \(projectName)"
        content.sound = .default
        content.categoryIdentifier = "SWARM_STARTED"

        sendNotification(identifier: "swarm-\(UUID().uuidString)", content: content)
    }

    private func sendNotification(identifier: String, content: UNMutableNotificationContent) {
        let request = UNNotificationRequest(
            identifier: identifier,
            content: content,
            trigger: nil  // Immediate delivery
        )

        UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
                print("NotificationManager: Failed to send notification: \(error)")
            }
        }
    }

    /// Clear all delivered notifications
    func clearAllNotifications() {
        UNUserNotificationCenter.current().removeAllDeliveredNotifications()
    }

    /// Clear notifications for a specific category
    func clearNotifications(category: String) {
        UNUserNotificationCenter.current().getDeliveredNotifications { notifications in
            let identifiers =
                notifications
                .filter { $0.request.content.categoryIdentifier == category }
                .map { $0.request.identifier }

            UNUserNotificationCenter.current().removeDeliveredNotifications(
                withIdentifiers: identifiers)
        }
    }
}

/// Extension to integrate notifications with AgentManager
extension AgentManager {

    /// Enable notifications for agent events
    func enableNotifications() {
        // This would be called in response to agent status changes
        // You'd add observers or callbacks to trigger notifications
    }

    func notifyIfTaskCompleted(agent: Agent) {
        guard agent.status == .completed,
            let task = agent.task
        else { return }

        NotificationManager.shared.notifyTaskCompleted(
            taskTitle: task.title,
            projectName: agent.projectName
        )
    }

    func notifyIfTaskFailed(agent: Agent) {
        guard agent.status == .failed,
            let task = agent.task
        else { return }

        NotificationManager.shared.notifyTaskFailed(
            taskTitle: task.title,
            projectName: agent.projectName
        )
    }

    func notifyIfConflictsDetected(for session: ConversationSession) {
        let conflicts = detectConflicts(for: session)

        for conflict in conflicts {
            NotificationManager.shared.notifyConflictDetected(
                fileName: conflict.file,
                agentCount: conflict.agentIds.count
            )
        }
    }
}

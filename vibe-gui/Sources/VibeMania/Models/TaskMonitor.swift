import Foundation

/// Monitors .vibemania/tasks.json for changes and updates task status in real-time
@Observable
final class TaskMonitor {
    private var fileDescriptor: Int32 = -1
    private var dispatchSource: DispatchSourceFileSystemObject?
    private var monitoredPath: String?

    var onTasksUpdated: (([AgentTask]) -> Void)?

    /// Start monitoring tasks.json in the given project directory
    func startMonitoring(projectPath: String) {
        stopMonitoring()  // Stop any existing monitoring

        let tasksPath = (projectPath as NSString).appendingPathComponent(".vibemania/tasks.json")
        monitoredPath = tasksPath

        // Create .vibemania directory if it doesn't exist
        let workDir = (projectPath as NSString).appendingPathComponent(".vibemania")
        try? FileManager.default.createDirectory(atPath: workDir, withIntermediateDirectories: true)

        // Create empty tasks.json if it doesn't exist
        if !FileManager.default.fileExists(atPath: tasksPath) {
            try? "[]".write(toFile: tasksPath, atomically: true, encoding: .utf8)
        }

        // Open file descriptor
        fileDescriptor = open(tasksPath, O_EVTONLY)
        guard fileDescriptor >= 0 else {
            print("TaskMonitor: Failed to open \(tasksPath)")
            return
        }

        // Create dispatch source for file monitoring
        dispatchSource = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fileDescriptor,
            eventMask: [.write, .extend],
            queue: DispatchQueue.global(qos: .userInteractive)
        )

        dispatchSource?.setEventHandler { [weak self] in
            self?.handleFileChange()
        }

        dispatchSource?.setCancelHandler { [weak self] in
            guard let fd = self?.fileDescriptor, fd >= 0 else { return }
            close(fd)
            self?.fileDescriptor = -1
        }

        dispatchSource?.resume()

        print("TaskMonitor: Started monitoring \(tasksPath)")
    }

    /// Stop monitoring
    func stopMonitoring() {
        dispatchSource?.cancel()
        dispatchSource = nil

        if fileDescriptor >= 0 {
            close(fileDescriptor)
            fileDescriptor = -1
        }

        monitoredPath = nil
    }

    private func handleFileChange() {
        guard let path = monitoredPath else { return }

        // Read and parse tasks.json
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: path)),
            let tasks = try? JSONDecoder().decode([AgentTask].self, from: data)
        else {
            return
        }

        // Notify on main thread
        DispatchQueue.main.async { [weak self] in
            self?.onTasksUpdated?(tasks)
        }
    }

    deinit {
        stopMonitoring()
    }
}

/// Helper extension for AgentManager to integrate task monitoring
extension AgentManager {

    /// Monitor tasks.json for a given session
    func startTaskMonitoring(for session: ConversationSession) {
        guard let projectPath = session.projectPath else { return }

        let monitor = TaskMonitor()
        monitor.onTasksUpdated = { [weak self] tasks in
            self?.updateSessionTasks(session: session, tasks: tasks)
        }
        monitor.startMonitoring(projectPath: projectPath)

        // Store monitor (you'd need to add a dictionary property to AgentManager)
        // self.taskMonitors[session.id] = monitor
    }

    private func updateSessionTasks(session: ConversationSession, tasks: [AgentTask]) {
        // Update existing tasks with new status
        for updatedTask in tasks {
            if let index = session.tasks.firstIndex(where: { $0.id == updatedTask.id }) {
                session.tasks[index] = updatedTask

                // Update agent status if task completed/failed
                if let agentId = updatedTask.assignedAgentId,
                    let agent = agents.first(where: { $0.id == agentId })
                {

                    if updatedTask.status == .completed && agent.status == .running {
                        agent.status = .completed
                        agent.endedAt = updatedTask.completedAt
                    } else if updatedTask.status == .failed && agent.status == .running {
                        agent.status = .failed
                        agent.endedAt = updatedTask.completedAt
                    }
                }
            } else {
                // New task discovered, add it
                session.tasks.append(updatedTask)
            }
        }
    }
}

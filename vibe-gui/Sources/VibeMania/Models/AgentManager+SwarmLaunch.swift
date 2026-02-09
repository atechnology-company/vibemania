import Foundation

extension AgentManager {

    /// Enhanced swarm launch with task parsing and parallel executors
    func launchSwarmWithTaskParsing(for session: ConversationSession) -> Agent {
        guard let projectPath = session.projectPath else {
            print("Error: No project path set for session")
            return createFailedAgent(for: session, error: "No project path set")
        }

        // Create planner agent
        let planner = Agent(
            projectId: session.projectId ?? UUID(),
            projectName: session.title,
            toolType: .claude,
            maxIterations: 1,
            role: .planner
        )
        agents.append(planner)
        session.agents.append(planner.id)

        // Add system message to conversation
        let systemMessage = ChatMessage(
            role: .system,
            content: "Starting VibeMania swarm with planner agent...",
            agentId: planner.id
        )
        session.messages.append(systemMessage)

        // Launch planner with callback to spawn executors
        startPlannerProcess(for: planner, session: session, projectPath: projectPath)

        return planner
    }

    private func startPlannerProcess(
        for planner: Agent, session: ConversationSession, projectPath: String
    ) {
        planner.status = .running
        planner.startedAt = Date()

        guard let scriptPath = findVibemaniaScript(from: projectPath) else {
            planner.status = .failed
            planner.logs += "Error: Could not find vibemania.sh\n"
            planner.endedAt = Date()
            return
        }

        let process = Process()
        let pipe = Pipe()
        let errorPipe = Pipe()

        // Build planner prompt - create goals.md if needed
        let workDir = (projectPath as NSString).appendingPathComponent(".vibemania")
        try? FileManager.default.createDirectory(atPath: workDir, withIntermediateDirectories: true)

        let goalsPath = (projectPath as NSString).appendingPathComponent("goals.md")

        // Create goals.md from first user message if it doesn't exist
        if !FileManager.default.fileExists(atPath: goalsPath),
            let firstMessage = session.messages.first(where: { $0.role == .user })
        {
            let goalsContent = """
                # Project Goals

                \(firstMessage.content)
                """
            try? goalsContent.write(toFile: goalsPath, atomically: true, encoding: .utf8)
        }

        process.executableURL = URL(fileURLWithPath: "/bin/bash")
        process.arguments = [
            scriptPath,
            "--tool", "claude",
            "--project-dir", projectPath,
            "1",  // Single iteration for planner
        ]
        process.currentDirectoryURL = URL(fileURLWithPath: projectPath)
        process.standardOutput = pipe
        process.standardError = errorPipe
        process.environment = ProcessInfo.processInfo.environment

        planner.process = process
        planner.outputPipe = pipe

        var accumulatedOutput = ""

        // Stream stdout and accumulate for parsing
        pipe.fileHandleForReading.readabilityHandler = { [weak planner] handle in
            let data = handle.availableData
            guard !data.isEmpty,
                let output = String(data: data, encoding: .utf8),
                let planner = planner
            else { return }

            DispatchQueue.main.async {
                planner.logs += output
                accumulatedOutput += output
            }
        }

        // Stream stderr
        errorPipe.fileHandleForReading.readabilityHandler = { [weak planner] handle in
            let data = handle.availableData
            guard !data.isEmpty,
                let output = String(data: data, encoding: .utf8),
                let planner = planner
            else { return }

            DispatchQueue.main.async {
                planner.logs += output
            }
        }

        // Handle termination - parse tasks and spawn executors
        process.terminationHandler = { [weak self] proc in
            guard let self = self else { return }

            DispatchQueue.main.async {
                pipe.fileHandleForReading.readabilityHandler = nil
                errorPipe.fileHandleForReading.readabilityHandler = nil

                if planner.status == .running {
                    planner.status = proc.terminationStatus == 0 ? .completed : .failed
                }
                planner.endedAt = Date()

                // Parse tasks from planner output
                let tasks = TaskParser.parseTasks(from: accumulatedOutput)

                // Add tasks to session
                for task in tasks {
                    session.tasks.append(task)
                }

                // Add planner message to conversation
                let plannerMessage = ChatMessage(
                    role: .planner,
                    content: "Generated \(tasks.count) task\(tasks.count == 1 ? "" : "s"):\n\n"
                        + tasks.map { "• \($0.title)" }.joined(separator: "\n"),
                    agentId: planner.id
                )
                session.messages.append(plannerMessage)

                // Launch executor agents for each task
                if tasks.count > 1 {
                    self.launchExecutors(for: tasks, session: session, projectPath: projectPath)
                } else if tasks.count == 1 {
                    // Single task - launch one executor
                    self.launchExecutor(for: tasks[0], session: session, projectPath: projectPath)
                }
            }
        }

        do {
            try process.run()
            DispatchQueue.main.async {
                planner.logs += "Planner started for project: \(session.title)\n"
                planner.logs += "Working directory: \(projectPath)\n\n"
            }
        } catch {
            DispatchQueue.main.async {
                planner.status = .failed
                planner.logs += "Failed to start planner: \(error.localizedDescription)\n"
                planner.endedAt = Date()
            }
        }
    }

    private func launchExecutors(
        for tasks: [AgentTask], session: ConversationSession, projectPath: String
    ) {
        let systemMessage = ChatMessage(
            role: .system,
            content: "Launching \(tasks.count) parallel executor agents..."
        )
        session.messages.append(systemMessage)

        for task in tasks {
            launchExecutor(for: task, session: session, projectPath: projectPath)
        }
    }

    private func launchExecutor(
        for task: AgentTask, session: ConversationSession, projectPath: String
    ) {
        let executor = Agent(
            projectId: session.projectId ?? UUID(),
            projectName: session.title,
            toolType: .claude,
            maxIterations: 10,
            role: .executor
        )
        executor.taskId = task.id
        executor.task = task

        agents.append(executor)
        session.agents.append(executor.id)

        // Update task status
        if let index = session.tasks.firstIndex(where: { $0.id == task.id }) {
            session.tasks[index].status = .running
            session.tasks[index].assignedAgentId = executor.id
            session.tasks[index].startedAt = Date()
        }

        // Create task-specific goals file
        let taskGoalsPath = (projectPath as NSString).appendingPathComponent(
            ".vibemania/task-\(task.id.uuidString).md")
        try? task.description.write(toFile: taskGoalsPath, atomically: true, encoding: .utf8)

        // For now, launch regular vibemania.sh for each task
        // TODO: Use vibemania-swarm.sh with task-specific plan file
        startProcess(
            for: executor,
            project: Project(
                id: session.projectId ?? UUID(),
                name: task.title,
                path: projectPath,
                toolType: .claude,
                maxIterations: 5
            )
        )

        // Add executor message to conversation
        let executorMessage = ChatMessage(
            role: .executor,
            content: "Starting work on: \(task.title)",
            agentId: executor.id
        )
        session.messages.append(executorMessage)
    }
}

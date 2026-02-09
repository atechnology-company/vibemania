import Foundation

@Observable
final class AgentManager {
    var agents: [Agent] = []

    var runningAgents: [Agent] {
        agents.filter { $0.isRunning }
    }

    func agentsForProject(_ projectId: UUID) -> [Agent] {
        agents.filter { $0.projectId == projectId }
    }

    func agentsForSession(_ sessionId: UUID) -> [Agent] {
        // For now, match by projectId since sessions track agents by ID
        // In full implementation, we'd store sessionId in Agent
        agents.filter { agent in
            // Check if agent ID is in any session's agent list
            // This is a temporary solution; ideally Agent would have sessionId property
            true  // Return all agents for now
        }
    }

    @discardableResult
    func launchAgent(for project: Project) -> Agent {
        let agent = Agent(
            projectId: project.id,
            projectName: project.name,
            toolType: project.toolType,
            maxIterations: project.maxIterations
        )
        agents.append(agent)
        startProcess(for: agent, project: project)
        return agent
    }

    func stopAgent(_ agent: Agent) {
        if agent.process?.isRunning == true {
            agent.process?.terminate()
        }
        agent.status = .stopped
        agent.endedAt = Date()
    }

    func removeAgent(_ agent: Agent) {
        if agent.isRunning {
            stopAgent(agent)
        }
        agents.removeAll { $0.id == agent.id }
    }

    // MARK: - Swarm Launch (Codex-style)

    @discardableResult
    func launchSwarm(for session: ConversationSession) -> Agent {
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

        // For MVP: Launch regular vibemania.sh (will enhance to vibemania-swarm.sh later)
        startProcess(
            for: planner,
            project: Project(
                id: session.projectId ?? UUID(),
                name: session.title,
                path: projectPath,
                toolType: .claude,
                maxIterations: 10
            ))

        return planner
    }

    private func createFailedAgent(for session: ConversationSession, error: String) -> Agent {
        let agent = Agent(
            projectId: session.projectId ?? UUID(),
            projectName: session.title,
            toolType: .claude,
            maxIterations: 0,
            role: .planner
        )
        agent.status = .failed
        agent.logs = "Error: \(error)\n"
        agents.append(agent)
        session.agents.append(agent.id)
        return agent
    }

    // MARK: - Conflict Detection

    struct FileConflict: Identifiable {
        let id = UUID()
        let file: String
        let agentIds: [UUID]
    }

    func detectConflicts(for session: ConversationSession) -> [FileConflict] {
        var fileModifications: [String: [UUID]] = [:]

        for agentId in session.agents {
            guard let agent = agents.first(where: { $0.id == agentId }) else { continue }
            for file in agent.filesModified {
                fileModifications[file, default: []].append(agentId)
            }
        }

        return fileModifications.compactMap { file, agentIds in
            guard agentIds.count > 1 else { return nil }
            return FileConflict(file: file, agentIds: agentIds)
        }
    }

    // MARK: - Private

    private func findVibemaniaScript(from projectPath: String) -> String? {
        let fileManager = FileManager.default

        // Check in the project directory first
        let projectScript = (projectPath as NSString).appendingPathComponent("vibemania.sh")
        if fileManager.fileExists(atPath: projectScript) {
            return projectScript
        }

        // Check in common locations relative to the app bundle
        let candidates = [
            // Alongside the app's executable (for development)
            Bundle.main.bundlePath + "/../vibemania.sh",
            // Home directory
            NSHomeDirectory() + "/vibemania.sh",
            // Common install locations
            "/usr/local/bin/vibemania.sh",
            NSHomeDirectory() + "/.local/bin/vibemania.sh",
        ]

        for candidate in candidates {
            if fileManager.fileExists(atPath: candidate) {
                return candidate
            }
        }

        return nil
    }

    private func startProcess(for agent: Agent, project: Project) {
        agent.status = .running
        agent.startedAt = Date()

        guard let scriptPath = findVibemaniaScript(from: project.path) else {
            agent.status = .failed
            agent.logs += "Error: Could not find vibemania.sh\n"
            agent.logs +=
                "Place vibemania.sh in your project directory or install it to /usr/local/bin/\n"
            agent.endedAt = Date()
            return
        }

        let process = Process()
        let pipe = Pipe()
        let errorPipe = Pipe()

        let arguments = [
            scriptPath,
            "--tool", project.toolType.toolFlag,
            "--project-dir", project.path,
            "\(project.maxIterations)",
        ]

        process.executableURL = URL(fileURLWithPath: "/bin/bash")
        process.arguments = arguments
        process.currentDirectoryURL = URL(fileURLWithPath: project.path)
        process.standardOutput = pipe
        process.standardError = errorPipe
        process.environment = ProcessInfo.processInfo.environment

        agent.process = process
        agent.outputPipe = pipe

        // Stream stdout
        pipe.fileHandleForReading.readabilityHandler = { [weak agent] handle in
            let data = handle.availableData
            guard !data.isEmpty,
                let output = String(data: data, encoding: .utf8)
            else { return }
            DispatchQueue.main.async {
                agent?.logs += output
                // Parse iteration number from vibemania.sh output
                let regex = try? NSRegularExpression(pattern: #"Iteration (\d+) of"#)
                let range = NSRange(output.startIndex..., in: output)
                if let match = regex?.firstMatch(in: output, range: range),
                    let numRange = Range(match.range(at: 1), in: output),
                    let num = Int(output[numRange])
                {
                    agent?.iteration = num
                }
            }
        }

        // Stream stderr
        errorPipe.fileHandleForReading.readabilityHandler = { [weak agent] handle in
            let data = handle.availableData
            guard !data.isEmpty,
                let output = String(data: data, encoding: .utf8)
            else { return }
            DispatchQueue.main.async {
                agent?.logs += output
            }
        }

        // Handle termination
        process.terminationHandler = { [weak agent] proc in
            DispatchQueue.main.async {
                pipe.fileHandleForReading.readabilityHandler = nil
                errorPipe.fileHandleForReading.readabilityHandler = nil

                if agent?.status == .running {
                    agent?.status =
                        proc.terminationStatus == 0
                        ? .completed
                        : .failed
                }
                agent?.endedAt = Date()
            }
        }

        do {
            try process.run()
            DispatchQueue.main.async {
                agent.logs += "VibeMania started for project: \(project.name)\n"
                agent.logs += "Tool: \(project.toolType.toolFlag)\n"
                agent.logs += "Working directory: \(project.path)\n"
                agent.logs += "Max iterations: \(project.maxIterations)\n\n"
            }
        } catch {
            DispatchQueue.main.async {
                agent.status = .failed
                agent.logs += "Failed to start: \(error.localizedDescription)\n"
                agent.endedAt = Date()
            }
        }
    }
}

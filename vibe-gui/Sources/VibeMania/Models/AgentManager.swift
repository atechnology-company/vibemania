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

    // MARK: - Private

    private func startProcess(for agent: Agent, project: Project) {
        agent.status = .running
        agent.startedAt = Date()

        let process = Process()
        let pipe = Pipe()
        let errorPipe = Pipe()

        var arguments: [String] = []
        let scriptPath: String

        switch project.toolType {
        case .vibe:
            scriptPath = (project.path as NSString)
                .appendingPathComponent("vibe.sh")
            arguments = ["--acknowledge-unsafe", "\(project.maxIterations)"]
        case .ralph:
            scriptPath = (project.path as NSString)
                .appendingPathComponent("ralph.sh")
            arguments = ["--tool", "amp", "\(project.maxIterations)"]
        case .ralphClaude:
            scriptPath = (project.path as NSString)
                .appendingPathComponent("ralph.sh")
            arguments = ["--tool", "claude", "\(project.maxIterations)"]
        }

        process.executableURL = URL(fileURLWithPath: "/bin/bash")
        process.arguments = [scriptPath] + arguments
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
                  let output = String(data: data, encoding: .utf8) else { return }
            DispatchQueue.main.async {
                agent?.logs += output
                let regex = try? NSRegularExpression(pattern: #"Iteration (\d+) of"#)
                let range = NSRange(output.startIndex..., in: output)
                if let match = regex?.firstMatch(in: output, range: range),
                   let numRange = Range(match.range(at: 1), in: output),
                   let num = Int(output[numRange]) {
                    agent?.iteration = num
                }
            }
        }

        // Stream stderr
        errorPipe.fileHandleForReading.readabilityHandler = { [weak agent] handle in
            let data = handle.availableData
            guard !data.isEmpty,
                  let output = String(data: data, encoding: .utf8) else { return }
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
                    agent?.status = proc.terminationStatus == 0
                        ? .completed
                        : .failed
                }
                agent?.endedAt = Date()
            }
        }

        do {
            try process.run()
            DispatchQueue.main.async {
                agent.logs += "[\(project.toolType.rawValue)] Started for project: \(project.name)\n"
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

import Foundation

struct TaskParser {

    /// Parse planner output for multi-task format
    /// Returns array of AgentTask objects extracted from <vibemania_tasks> block
    static func parseTasks(from plannerOutput: String) -> [AgentTask] {
        var tasks: [AgentTask] = []

        // Check if planner output contains multi-task format
        guard plannerOutput.contains("<vibemania_tasks") else {
            // Single task mode - create one task from entire output
            let task = AgentTask(
                title: extractSingleTaskTitle(from: plannerOutput),
                description: plannerOutput,
                planFile: "plan.md",
                filesAffected: extractFilesAffected(from: plannerOutput)
            )
            return [task]
        }

        // Extract max_parallel attribute
        let maxParallel = extractMaxParallel(from: plannerOutput)

        // Split into individual tasks by "### Task N:" pattern
        let taskPattern = #"### Task (\d+): ([^\n]+)([\s\S]*?)(?=### Task \d+:|</vibemania_tasks>)"#
        guard let regex = try? NSRegularExpression(pattern: taskPattern, options: []) else {
            return []
        }

        let nsString = plannerOutput as NSString
        let matches = regex.matches(
            in: plannerOutput, range: NSRange(location: 0, length: nsString.length))

        for match in matches {
            guard match.numberOfRanges >= 4 else { continue }

            let taskNumberRange = match.range(at: 1)
            let titleRange = match.range(at: 2)
            let contentRange = match.range(at: 3)

            guard let taskNumberStr = Range(taskNumberRange, in: plannerOutput),
                let titleStrRange = Range(titleRange, in: plannerOutput),
                let contentStrRange = Range(contentRange, in: plannerOutput)
            else {
                continue
            }

            let taskNumber = String(plannerOutput[taskNumberStr])
            let title = String(plannerOutput[titleStrRange]).trimmingCharacters(in: .whitespaces)
            let content = String(plannerOutput[contentStrRange]).trimmingCharacters(
                in: .whitespacesAndNewlines)

            let filesAffected = extractFilesAffected(from: content)

            let task = AgentTask(
                title: title,
                description: content,
                planFile: "plan-\(taskNumber).md",
                filesAffected: filesAffected
            )

            tasks.append(task)
        }

        return tasks
    }

    /// Extract max_parallel attribute from <vibemania_tasks> tag
    private static func extractMaxParallel(from text: String) -> Int {
        let pattern = #"<vibemania_tasks[^>]*max_parallel="(\d+)""#
        guard let regex = try? NSRegularExpression(pattern: pattern, options: []),
            let match = regex.firstMatch(
                in: text, range: NSRange(location: 0, length: text.utf16.count)),
            match.numberOfRanges >= 2,
            let range = Range(match.range(at: 1), in: text),
            let maxParallel = Int(text[range])
        else {
            return 3  // Default
        }
        return maxParallel
    }

    /// Extract single task title from traditional format
    private static func extractSingleTaskTitle(from text: String) -> String {
        // Look for "### Task Title" or "## Task:" pattern
        let patterns = [
            #"### Task Title\s*\n([^\n]+)"#,
            #"## Task:\s*([^\n]+)"#,
            #"^# ([^\n]+)"#,
        ]

        for pattern in patterns {
            if let regex = try? NSRegularExpression(
                pattern: pattern, options: [.anchorsMatchLines]),
                let match = regex.firstMatch(
                    in: text, range: NSRange(location: 0, length: text.utf16.count)),
                match.numberOfRanges >= 2,
                let range = Range(match.range(at: 1), in: text)
            {
                return String(text[range]).trimmingCharacters(in: .whitespaces)
            }
        }

        return "Task"
    }

    /// Extract files affected from task description
    /// Looks for "#### Files Affected" section
    private static func extractFilesAffected(from text: String) -> [String] {
        let pattern = #"####\s*Files? Affected[:\s]*\n((?:[-*]\s+.+\n?)+)"#
        guard let regex = try? NSRegularExpression(pattern: pattern, options: [.caseInsensitive]),
            let match = regex.firstMatch(
                in: text, range: NSRange(location: 0, length: text.utf16.count)),
            match.numberOfRanges >= 2,
            let range = Range(match.range(at: 1), in: text)
        else {
            return []
        }

        let filesSection = String(text[range])

        // Extract individual file paths (lines starting with - or *)
        let filePattern = #"[-*]\s+(.+)"#
        guard let fileRegex = try? NSRegularExpression(pattern: filePattern, options: []) else {
            return []
        }

        let matches = fileRegex.matches(
            in: filesSection, range: NSRange(location: 0, length: filesSection.utf16.count))

        return matches.compactMap { match in
            guard match.numberOfRanges >= 2,
                let range = Range(match.range(at: 1), in: filesSection)
            else {
                return nil
            }
            return String(filesSection[range]).trimmingCharacters(in: .whitespaces)
        }
    }

    /// Split planner output into individual plan files
    /// Returns dictionary of planFileName -> planContent
    static func splitIntoFiles(tasks: [AgentTask], fullPlannerOutput: String) -> [String: String] {
        var planFiles: [String: String] = [:]

        for task in tasks {
            // For now, just use the task description as the plan content
            // In a more sophisticated implementation, we could extract the full task section
            let planContent = """
                # \(task.title)

                \(task.description)
                """

            planFiles[task.planFile] = planContent
        }

        return planFiles
    }
}

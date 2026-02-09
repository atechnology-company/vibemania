import Foundation

struct LogParser {

    /// Extract files modified from agent output logs
    /// Looks for patterns like:
    /// - "Editing file: path/to/file.swift"
    /// - "Writing file: path/to/file.swift"
    /// - "Modified: path/to/file.swift"
    /// - Git diff output
    static func extractModifiedFiles(from logs: String) -> [String] {
        var files = Set<String>()

        // Pattern 1: Tool output "Editing file:", "Writing file:", "Reading file:" etc.
        let toolPatterns = [
            #"(?:Editing|Writing|Creating|Modifying|Updated?)\s+(?:file:?\s+)?([^\s\n]+\.[a-zA-Z]{2,5})"#,
            #"File\s+(?:path|name)?:?\s+([^\s\n]+\.[a-zA-Z]{2,5})"#,
        ]

        for pattern in toolPatterns {
            if let regex = try? NSRegularExpression(pattern: pattern, options: [.caseInsensitive]) {
                let matches = regex.matches(
                    in: logs, range: NSRange(location: 0, length: logs.utf16.count))
                for match in matches {
                    if match.numberOfRanges >= 2,
                        let range = Range(match.range(at: 1), in: logs)
                    {
                        let file = String(logs[range]).trimmingCharacters(in: .whitespaces)
                        files.insert(file)
                    }
                }
            }
        }

        // Pattern 2: Git diff output
        // modified:   path/to/file.swift
        let gitPattern = #"modified:\s+([^\n]+)"#
        if let regex = try? NSRegularExpression(pattern: gitPattern, options: []) {
            let matches = regex.matches(
                in: logs, range: NSRange(location: 0, length: logs.utf16.count))
            for match in matches {
                if match.numberOfRanges >= 2,
                    let range = Range(match.range(at: 1), in: logs)
                {
                    let file = String(logs[range]).trimmingCharacters(in: .whitespaces)
                    files.insert(file)
                }
            }
        }

        // Pattern 3: Vibemania progress.md updates
        // "## Files changed: src/main.swift, src/utils.swift"
        let progressPattern = #"##\s*Files?\s+changed?:\s*([^\n]+)"#
        if let regex = try? NSRegularExpression(
            pattern: progressPattern, options: [.caseInsensitive])
        {
            let matches = regex.matches(
                in: logs, range: NSRange(location: 0, length: logs.utf16.count))
            for match in matches {
                if match.numberOfRanges >= 2,
                    let range = Range(match.range(at: 1), in: logs)
                {
                    let filesStr = String(logs[range])
                    // Split by comma
                    let fileList = filesStr.split(separator: ",").map {
                        String($0).trimmingCharacters(in: .whitespaces)
                    }
                    files.formUnion(fileList)
                }
            }
        }

        // Pattern 4: Common file paths in logs (conservative - must have extension)
        // Look for paths like src/file.swift, lib/utils.ts, etc.
        let pathPattern = #"(?:^|\s)((?:[a-zA-Z0-9_-]+/)+[a-zA-Z0-9_-]+\.[a-zA-Z]{2,5})(?:\s|$|:)"#
        if let regex = try? NSRegularExpression(pattern: pathPattern, options: [.anchorsMatchLines])
        {
            let matches = regex.matches(
                in: logs, range: NSRange(location: 0, length: logs.utf16.count))
            for match in matches {
                if match.numberOfRanges >= 2,
                    let range = Range(match.range(at: 1), in: logs)
                {
                    let file = String(logs[range]).trimmingCharacters(in: .whitespaces)
                    // Only include if it looks like a real path (not URL, not random string)
                    if !file.contains("http") && file.contains("/") {
                        files.insert(file)
                    }
                }
            }
        }

        return Array(files).sorted()
    }

    /// Extract task completion status from logs
    /// Looks for patterns like "Task completed", "COMPLETE", etc.
    static func extractTaskStatus(from logs: String) -> TaskStatus? {
        let completionPatterns = [
            #"(?i)task\s+completed?"#,
            #"(?i)successfully?\s+completed?"#,
            #"<vibemania>COMPLETE</vibemania>"#,
            #"(?i)all\s+done"#,
        ]

        for pattern in completionPatterns {
            if let regex = try? NSRegularExpression(pattern: pattern, options: []),
                regex.firstMatch(in: logs, range: NSRange(location: 0, length: logs.utf16.count))
                    != nil
            {
                return .completed
            }
        }

        let failurePatterns = [
            #"(?i)task\s+failed"#,
            #"(?i)error:"#,
            #"(?i)failed\s+to"#,
        ]

        for pattern in failurePatterns {
            if let regex = try? NSRegularExpression(pattern: pattern, options: []),
                regex.firstMatch(in: logs, range: NSRange(location: 0, length: logs.utf16.count))
                    != nil
            {
                return .failed
            }
        }

        return nil  // Still running
    }

    /// Extract iteration number from logs
    /// Looks for "Iteration X of Y" pattern
    static func extractIteration(from logs: String) -> Int? {
        let pattern = #"Iteration\s+(\d+)\s+of"#
        guard let regex = try? NSRegularExpression(pattern: pattern, options: []),
            let match = regex.matches(
                in: logs, range: NSRange(location: 0, length: logs.utf16.count)
            ).last,
            match.numberOfRanges >= 2,
            let range = Range(match.range(at: 1), in: logs),
            let iteration = Int(logs[range])
        else {
            return nil
        }
        return iteration
    }
}

---
name: vibemania
description: "Set up VibeMania for a project by creating goals.md. Use when starting a new project with VibeMania, setting up autonomous development, or configuring project goals. Triggers on: create goals, vibemania goals, set up vibemania, configure vibemania, vibemania setup."
user-invocable: true
---

# VibeMania Goals Setup

Create a well-structured `goals.md` file that VibeMania's planner can use to drive autonomous development.

---

## The Job

1. Receive a project description from the user
2. Ask 3-5 essential clarifying questions (with lettered options)
3. Generate a structured `goals.md` based on answers
4. Save to `goals.md` in the project root

**Important:** Do NOT start implementing. Just create the goals file.

---

## Step 1: Clarifying Questions

Ask only critical questions where the initial prompt is ambiguous. Focus on:

- **End State:** What does the finished project look like?
- **Scope:** What should it NOT do?
- **Tech Stack:** Any required languages, frameworks, or tools?
- **Quality Bar:** What quality checks must pass?

### Format Questions Like This:

```
1. What is the primary goal of this project?
   A. Build a new application from scratch
   B. Add a major feature to an existing app
   C. Refactor or improve existing code
   D. Other: [please specify]

2. What tech stack should be used?
   A. Keep the existing stack (detected: [stack])
   B. Node.js / TypeScript
   C. Python
   D. Other: [please specify]

3. What quality requirements apply?
   A. Tests must pass
   B. Type checking must pass
   C. Linting must pass
   D. All of the above
```

This lets users respond with "1A, 2B, 3D" for quick iteration.

---

## Step 2: Goals Structure

Generate `goals.md` with these sections:

### 1. What We Are Building
One or two paragraphs describing the project's end state. Be specific enough that an AI planner can determine when goals are achieved.

### 2. Specific Goals
A checklist of concrete, verifiable goals. Each goal should be:
- **Small enough** to achieve in 1-3 VibeMania iterations
- **Specific** enough to verify ("Add user auth" is bad; "Add JWT-based login with email/password" is good)
- **Ordered** by dependency (database before backend before UI)

Format:
```markdown
- [ ] Goal 1: Add users table with email, password_hash, created_at columns
- [ ] Goal 2: Create /api/auth/login endpoint returning JWT
- [ ] Goal 3: Add login form at /login using existing form components
```

### 3. Constraints
Technical and style constraints:
- Required technologies or frameworks
- Patterns to follow from existing code
- Things to avoid

### 4. Quality Requirements
Specific commands or checks that must pass:
- Test commands (e.g., `npm test`, `pytest`, `cargo test`)
- Type checking (e.g., `npm run typecheck`, `mypy`)
- Linting (e.g., `npm run lint`, `ruff check`)

---

## Output

- **Format:** Markdown (`.md`)
- **Location:** Project root
- **Filename:** `goals.md`

---

## Example

```markdown
# Project Goals

## What We Are Building
A task management API with a React frontend. Users can create, edit, delete, and
filter tasks by priority and status. The backend is Express + PostgreSQL, the
frontend is React + TypeScript with Tailwind CSS.

## Specific Goals
- [ ] Add priority field (high/medium/low) to tasks table with default 'medium'
- [ ] Create GET /api/tasks endpoint with priority and status query filters
- [ ] Create POST /api/tasks endpoint that accepts priority field
- [ ] Add priority badge component (red=high, yellow=medium, gray=low)
- [ ] Display priority badges on task cards in the task list
- [ ] Add priority filter dropdown to task list header
- [ ] Add priority selector to task edit modal

## Constraints
- Use existing Express router pattern in src/routes/
- Follow existing React component patterns in src/components/
- Use Tailwind utility classes, no custom CSS
- PostgreSQL migrations use knex

## Quality Requirements
- npm run typecheck must pass
- npm run lint must pass
- npm test must pass
```

---

## Checklist

Before saving goals.md:

- [ ] Asked clarifying questions with lettered options
- [ ] Incorporated user's answers
- [ ] Goals are small and specific (1-3 iterations each)
- [ ] Goals are ordered by dependency
- [ ] Quality requirements include specific commands
- [ ] Saved to project root as `goals.md`

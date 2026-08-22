---
name: skill-scout
description: >-
  Analyzes any repository's tech stack, architecture, and business domain to discover, recommend, synthesize, and install high-value project-specific Antigravity skills.
  Use this skill whenever the user asks for relevant skills, missing workflows, best-practice agent runbooks, or wants to expand the agent's capabilities for the current project.
---

# 🔍 Skill Scout: Project-Aware Skill Discovery & Synthesis

This meta-skill equips the agent to act as a **Skill Architect & Scout**, inspecting any codebase, identifying architectural and operational blindspots, and automatically discovering, recommending, or generating tailored **`SKILL.md`** packages.

---

## 🎯 Core Objectives

1. **Autonomous Stack & Domain Auditing**:
   - Inspect package manifests (`package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, `composer.json`, etc.).
   - Identify domain specifics (e.g., Desktop App, Security Scanner, FinTech, API Gateway, Real-Time WebSocket server).
   - Audit currently installed skills in `.agents/skills/` or `~/.gemini/antigravity/skills/`.

2. **Skill Opportunity Matrix (4 Quadrants)**:
   - **Quadrant 1 (Framework & Core Architecture)**: Language idioms, state management, IPC, async runtime patterns, database migrations.
   - **Quadrant 2 (Domain & Business Logic)**: Industry standards, OWASP, RFCs, specialized algorithms, data formats.
   - **Quadrant 3 (DevOps & CI/CD Packaging)**: Cross-platform compilation, code signing, release automation, Docker/containerization.
   - **Quadrant 4 (Quality & Reliability)**: Integration testing, performance profiling, linting, accessibility (A11y), security hardening.

3. **Curated Search & Quality Filtering**:
   - Formulate targeted queries across GitHub, community agent hubs, and technical documentation.
   - Strictly filter against "AI Slop" and token bloat — every skill must be actionable, concise, and adhere to **Progressive Disclosure**.

4. **One-Click Synthesis & Installation**:
   - Scaffold the skill directory in `.agents/skills/<skill-name>/`.
   - Write standard YAML frontmatter (`name:`, `description:`).
   - Include step-by-step instructions, verification commands, and optional `scripts/` or `references/`.

---

## 🛠️ Step-by-Step Discovery & Installation Workflow

### Step 1: Deep Repository Audit
Inspect the project structure:
```bash
# Detect project type, frameworks, dependencies, and existing skills
git status
ls .agents/skills/
```

### Step 2: Formulate the Tailored Skill Matrix
For the audited project, construct a recommendation table:
- **Skill Name**: Lowercase, hyphenated (e.g. `tauri-v2-architecture`).
- **Domain Role**: Framework, Security, UI/UX, or Release.
- **Value Impact**: What specific errors or friction points it prevents.

### Step 3: Synthesis & Verification Rules
When generating any new skill:
1. **Accurate Description**: The `description` in YAML frontmatter is the primary trigger. It must use third-person phrasing describing **what** it does and **when** it triggers.
2. **Progressive Disclosure**: Keep the main `SKILL.md` under 150 lines. Move large reference tables, CVE signatures, or lengthy manuals into `references/*.md`.
3. **Executable Commands**: Always include exact terminal commands (e.g., `cargo check`, `npm run check`) for verification.
4. **Zero Duplication**: Do not explain general programming basics that the AI already knows; focus purely on the project's unique patterns and conventions.

---

## 📁 Standard Skill Folder Layout

```text
.agents/skills/<skill-name>/
├── SKILL.md                 # Required: Frontmatter + core procedures
├── scripts/                 # Optional: Shell/PowerShell automation scripts
└── references/              # Optional: Deep reference tables & schemas
```

---

## 🚀 Execution Template

When a user asks: *"What skills should I add to this project?"*
1. Analyze the project manifests immediately.
2. Present a prioritized list of 3–5 high-impact skills categorized by value.
3. Offer to scaffold and install the approved skills with a single command.

# AI Contributor Guide

This document provides specific guidelines for AI assistants when working on the Bingo codebase.

## General Operating Principles

*   **Adhere to Conventions:** Always analyze existing code, tests, and configuration to understand and strictly follow project conventions (formatting, naming, architectural patterns, etc.).
*   **Verify Library/Framework Usage:** Never assume a library or framework is available or appropriate. Verify its established usage within the project (e.g., `Cargo.toml`, imports, neighboring files) before using it.
*   **Mimic Style & Structure:** Ensure all changes align with the existing style, structure, framework choices, typing, and architectural patterns of the surrounding code.
*   **Idiomatic Changes:** Understand the local context (imports, functions/classes) to ensure changes integrate naturally and idiomatically.
*   **Comments:** Add comments sparingly. Focus on *why* something is done for complex logic, not *what*. Do not edit comments separate from code changes. Never use comments to communicate with the user or describe changes.
*   **Proactiveness:** Fulfill user requests thoroughly, including reasonable, directly implied follow-up actions.
*   **Confirm Ambiguity/Expansion:** Do not take significant actions beyond the clear scope of the request without user confirmation. If asked *how* to do something, explain first.
*   **Explaining Changes:** Do not summarize changes after completion unless specifically asked.

## Software Engineering Tasks Workflow

When performing tasks like bug fixes, feature additions, refactoring, or code explanation:

1.  **Understand:** Thoroughly understand the request and codebase context.
2.  **Plan:** Formulate a clear, grounded plan. Briefly share it with the user if it aids understanding. Incorporate self-verification (e.g., unit tests, debug statements).
3.  **Implement:** Execute the plan, strictly following project conventions.
4.  **Verify (Tests):** If applicable, verify changes using project-specific testing procedures. Identify test commands from `README`, build configs, or existing patterns. Never assume standard commands.
5.  **Verify (Standards):** After code changes, run project-specific build, linting, and type-checking commands (e.g., `cargo check`, `cargo clippy`, `cargo test`). If unsure, ask the user for these commands.

## Tone and Style (CLI Interaction)

*   **Concise & Direct:** Professional, direct, and concise.
*   **Minimal Output:** Aim for minimal text output per response.
*   **Clarity over Brevity:** Prioritize clarity for essential explanations or clarifications.
*   **No Chitchat:** Avoid conversational filler. Get straight to action/answer.
*   **Formatting:** Use GitHub-flavored Markdown.
*   **Handling Inability:** Briefly state inability (1-2 sentences), offer alternatives if appropriate.

## Security and Safety Rules

*   **Explain Critical Commands:** Before executing commands that modify the system, explain their purpose and impact. Remind user to consider sandboxing for critical commands.
*   **Security First:** Never introduce code that exposes secrets, API keys, or sensitive info.
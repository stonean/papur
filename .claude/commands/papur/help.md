---
description: Display an overview of the pipeline and its slash commands.
---

# Help

Display an overview of the pipeline and how to use its slash commands.

## Instructions

Print the following guide exactly (do not scan files or run commands):

---

## papur — Spec-Driven Development Pipeline

papur is a set of slash commands that guide features from idea to implementation through a structured pipeline.

### Pipeline States

```text
draft → clarified → planned → in-progress → done
```

Two back-edges keep the lifecycle honest:

- `/papur:ask` reverts a `clarified`, `planned`, or `in-progress` spec to `draft` when a new open question surfaces — `draft` is the only status that tolerates open questions. The next `/papur:clarify` resolves the question and the spec advances forward again.
- `/papur:ask` reverts a `done` spec to `in-progress` when a new scenario is added (the scenario route) — the scenario captures the change, the spec evolves with it.

Each feature lives in `specs/NNN-feature-name/` and progresses through these states by running the corresponding command.

### Commands

#### Pipeline (advance state)

<!-- generated:commands-pipeline:start -->

| Command | Pipeline Gate | Description |
| --- | --- | --- |
| `/papur:specify` | → draft | Create a new feature spec. |
| `/papur:clarify` | draft → clarified | Resolve open questions and advance a spec from draft to clarified. |
| `/papur:plan` | clarified → planned | Create a technical plan and task breakdown for a clarified spec. |
| `/papur:implement` | planned → in-progress → done | Execute implementation tasks for the targeted feature. |
| `/papur:review` | blocks `done` (MUST violations) | Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations. |
| `/papur:analyze` | — | Audit artifacts against each other — spec, plan, tasks, scenarios, frontmatter, dependencies, rule IDs. Read-only. |

<!-- generated:commands-pipeline:end -->

#### Refine

<!-- generated:commands-refine:start -->

| Command | Description |
| --- | --- |
| `/papur:ask` | Add a question or a scenario to the targeted spec (classifier-driven). |

<!-- generated:commands-refine:end -->

#### Brownfield (absorb existing reality)

<!-- generated:commands-brownfield:start -->

| Command | Description |
| --- | --- |
| `/papur:log` | Record a raw item to the inbox. |
| `/papur:groom` | Walk the inbox and route each item to its proper home. |

<!-- generated:commands-brownfield:end -->

#### Orient

<!-- generated:commands-orient:start -->

| Command | Description |
| --- | --- |
| `/papur:target` | Set the working feature (and optionally scenario) for this session. |
| `/papur:status` | Display the pipeline dashboard for all feature specs. |
| `/papur:help` | Display an overview of the pipeline and its slash commands. |

<!-- generated:commands-orient:end -->

#### Bootstrap (one-time per project)

<!-- generated:commands-bootstrap:start -->

| Command | Description |
| --- | --- |
| `/govern` | Adopt or update govern in an existing project. |
| `/papur:configure` | Configure settings.local.json with permissions for slash commands. |

<!-- generated:commands-bootstrap:end -->

### Typical Session

```text
/papur:configure                 # first time only
/papur:status                    # see where everything stands
/papur:target 000                # pick a feature to work on
/papur:clarify                   # resolve open questions
/papur:plan                      # generate implementation plan
/papur:implement                 # write the code
```

### Key Concepts

- **Session target** — The feature you're currently working on, stored in `.claude/papur-session.json`. Most commands operate on the target by default.
- **Dependencies** — Features declare dependencies in their spec. A feature is blocked until its dependencies reach `clarified` or later.
- **Artifacts** — Each feature directory can contain `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, and a `scenarios/` subdirectory.
- **Scenarios** — A scenario is a spec at a lower level of abstraction. Scenarios live in `specs/NNN-feature/scenarios/slug.md` and capture bugs, edge cases, and detailed behavior. Each scenario gets a linked task in `tasks.md`.
- **Bug decision tree** — When a bug is reported: (1) no spec → write the spec first, (2) spec is ambiguous → fix the spec, (3) spec is clear → add a scenario.
- **Inbox** — `specs/inbox.md` is a temporary inbox for known issues. Items are recorded with `/papur:log` and groomed into specs or scenarios with `/papur:groom`.
- **Finish before moving on** — Prefer completing a feature through the full pipeline before starting the next. Depth-first keeps context focused.

---

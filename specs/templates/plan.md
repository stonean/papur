# {NNN} — {Feature Name} Plan

Implements [{NNN} — {Feature Name}](spec.md).

<!-- This plan records the design as it currently stands, in the sections below
     and no others — they are its design record (§plan-phase). When
     implementation changes a decision, edit that decision's entry to say what
     is now true; never append a note that it changed, because git history
     already keeps what the plan said when. Everything else has a home that is
     not the plan: verification evidence in the commit that lands the task,
     contributor knowledge where §drift-prevention's "Shared knowledge stays
     in git" routes it, a finding's decision where the finding was
     dispositioned, and handoff notes on the pending task in tasks.md. `/papur:analyze`
     reports any other `##` section as outside the design record. -->

## Overview

<!-- Brief summary of the implementation approach and key technical decisions. -->

## Technical Decisions

<!-- List decisions made during planning and their rationale. Example:

### Session storage

Sessions are stored in PostgreSQL for durability and cached in Valkey for speed.
Alternative considered: Valkey-only — rejected because sessions would be lost on cache eviction.

-->

## Affected Files

<!-- Planning aid only — `/papur:implement` derives the runtime write boundary
     from `git diff` against the spec dir's first commit. List files you expect
     to create or modify so reviewers can sanity-check scope. The list does not
     need to be exhaustive; implement-time additions surface naturally. Example:

| File | Action | Purpose |
| --- | --- | --- |
| `src/auth/session` | Create | Session management logic |
| `src/middleware/auth` | Create | Auth middleware |
| `migrations/20250228_create_sessions` | Create | Sessions table |

-->

## Data Model

<!-- Optional — omit this section when the feature introduces or modifies no
     domain entity or data structure. Point at data-model.md and summarize
     what it defines. -->

## Trade-offs

<!-- What was considered and rejected, with the reason for each, and the known
     limitations of the chosen design. -->

## Open Questions Resolved

<!-- Optional — omit this section when no question the spec resolved needs a
     plan-level answer. How the plan addresses each one that does. -->

## Cross-spec impact

<!-- Optional — omit this section when no other spec is affected. Each spec
     this change affects, what it owes, and how that is discharged
     (§cross-spec-impact). -->

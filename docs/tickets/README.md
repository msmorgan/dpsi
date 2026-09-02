# Tickets

A ticket's status is the folder it lives in:

| Folder | Meaning |
|---|---|
| `planned/` | Committed to for 0.1 |
| `maybe/` | Deferred or design-gated; open only when there is a reason |
| `wip/` | Claimed and in progress |
| `done/` | Integrated into the default line |

Tickets move `planned/maybe → wip → done`. `kata claim <slug>` moves the
ticket into `wip/` and provisions the feature workspace `.workspaces/<slug>`;
`kata integrate <slug>` folds the finished work into the default line and moves
the ticket to `done/`. Those moves belong to jj-kata's bundled folder-Kanban
driver (configured in `kata.toml`); never move ticket files by hand.

## Writing a ticket

The frontmatter `needs: [...]` list is the only prescribed part: the slugs that
must reach `done/` before this one can be worked. A ticket tagged
`design: true` needs a design conversation with the user before implementation.
The body describes an actionable change and records the design decisions
already made that bear on it. Length and structure are free.

## Querying the graph

`needs:` is machine-read graph data. Use `kata kanban` rather than opening
dependency tickets to work out status by hand:

| Command | What it prints |
|---|---|
| `board` | Every ticket grouped by column |
| `ready` | Claimable tickets whose every need is in `done/` |
| `blocked` | Claimable tickets with an unmet need, and which needs |
| `order` | Every unfinished ticket in topological order |
| `graph <slug>` | Recursive upstream needs and direct downstream dependents |
| `needs <slug>` | Direct needs only |
| `check` | Duplicates, dangling needs, cycles; exits 1 on problems |

Run `check` after editing any `needs:`.

## Working an item

1. From `default`: `kata claim <slug>`.
2. `cd .workspaces/<slug>` and do the work there. Never do feature work in
   `default`.
3. Close the work with `jj commit -m "..."` so the feature `@` is empty.
4. From `default`: `kata integrate <slug>`, then `kata drop <slug>`.

Never run `kata refresh --all`; it is human-only.

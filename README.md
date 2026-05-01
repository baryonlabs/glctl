# glctl

`glctl` is the Generation Lineage Control Tool for Nautilus. It is the Git-like
local CLI for recording, inspecting, validating, and pushing AI-agent evolution
history.

Nautilus is the meta-loop and system-of-record for AI agent work. Paperclip is one frontend (control plane / board UI); other frontends are possible. `glctl` and `glhub` serve Nautilus directly — they do not depend on any single frontend.

The paired service is `glhub`: a separate web server that acts like a GitHub-like
thinking space for evolution documents. `glctl` owns the local lineage store;
`glhub` presents and stores pushed snapshots.

## Concepts

- **Generation**: one recorded evolution step.
- **Relation**: an edge between generations, usually `evolved_from`.
- **Company scope**: every read and write is scoped by `GLCTL_COMPANY_ID`.
- **Evolution document**: a readable before/after record built from generation
  data, retrospective notes, score deltas, gains/losses, and downstream children.
- **Retrospective**: first-class memory inside a generation: what not to repeat,
  what to do, skills created, bugs fixed, and cases that changed judgment.

## Storage

Default storage root:

```sh
./data/glctl
```

Effective layout:

```text
data/glctl/
└── companies/
    └── {company_id}/
        └── generations/
            ├── gen-YYYYMMDD-NNN.yaml
            └── relations/
                └── {from}-{to}.yaml
```

Required scope:

```sh
export GLCTL_COMPANY_ID=demo_company
```

Optional data root:

```sh
export GLCTL_DATA_DIR=/path/to/data/glctl
```

Company ids may contain ASCII letters, digits, `_`, and `-`.

## Build

```sh
cargo build --release
```

The release binary is:

```sh
./target/release/glctl
```

## Commands

### init

Initialize the current company-scoped lineage repository.

```sh
glctl init --json
```

### new

Create a generation.

```sh
glctl new \
  --soul "Capture retrospective as first-class evolution memory" \
  --parent gen-20260427-002 \
  --gains "Evolution document now shows rules, skills, bugs, and cases" \
  --losses "Raw score alone is no longer enough context" \
  --note "A generation is only useful if the next agent can see what changed judgment." \
  --score 0.89 \
  --tag retrospective \
  --do-not "Hide lessons only in chat history" \
  --do "Record lessons in the generation itself" \
  --skill "glhub evolution document" \
  --bug-fixed "Retrospective context was missing from generation records" \
  --case-json '[{"name":"Shortify prompts","impact":"Output contracts and fallbacks shaped the design."}]'
```

`glctl new` prints only the new generation id to stdout.

### show

Show one generation as canonical YAML or JSON.

```sh
glctl show gen-20260427-003
glctl show gen-20260427-003 --json
```

Paperclip and glhub use `show --json` instead of reparsing YAML themselves.

### list

List generations in reverse chronological order.

```sh
glctl list
glctl list --json
glctl list --limit 10
```

### lineage

Show lineage nodes and edges.

```sh
glctl lineage --json
glctl lineage --json --from gen-20260427-003
```

### graph

Render Mermaid flowchart DSL.

```sh
glctl graph
```

### status

Summarize the current repository.

```sh
glctl status --json
```

Includes generation count, relation count, seeds, heads, latest generation, best
successful generation, and dangling parent count.

### fsck

Check repository integrity.

```sh
glctl fsck --json
```

Checks generation id shape, score range, missing parents, missing relation edges,
relation endpoints, and config patch limits.

### push

Push the current company-scoped snapshot to a glhub server.

```sh
glctl push --remote http://127.0.0.1:3201
```

Remote resolution order:

1. `--remote`
2. `GLHUB_URL`
3. `http://127.0.0.1:3201`

Payload shape:

```json
{
  "schema_version": "glhub-push/v1",
  "company_id": "demo_company",
  "pushed_at": "2026-04-27T18:23:15Z",
  "status": {
    "generation_count": 3,
    "relation_count": 2,
    "latest_generation_id": "gen-...",
    "best_generation_id": "gen-..."
  },
  "generations": [],
  "relations": []
}
```

## glhub

`glhub` is a separate server in `../glhub`.

Run it:

```sh
pnpm --filter @paperclipai/glhub build
node ../glhub/dist/index.js
```

Open:

```text
http://127.0.0.1:3201
```

Current web view:

- header company repository selector
- language selector: English / Korean
- metrics
- lineage graph
- full-width side-by-side evolution documents: `Evolution document 1 | Evolution document 2`
- comment / edit proposal composer

Selecting a generation compares:

```text
parent generation | selected generation
```

Comments and edit proposals are saved as child generations so the original
evolution document remains auditable.

## Push Storage

`glhub` stores pushed snapshots through `/api/push`.

If R2 environment variables are present, snapshots are written to R2:

```sh
GLHUB_R2_BUCKET=...
GLHUB_R2_ENDPOINT=...
GLHUB_R2_ACCESS_KEY_ID=...
GLHUB_R2_SECRET_ACCESS_KEY=...
GLHUB_R2_PREFIX=glhub
```

If R2 is not configured, glhub uses local fallback storage:

```text
data/glhub/pushes/{company_id}/{push_id}.json
data/glhub/pushes/{company_id}/latest.json
```

## Verification

```sh
cargo fmt -- --check
cargo test
cargo build --release
pnpm --filter @paperclipai/glhub typecheck
pnpm --filter @paperclipai/glhub build
```

Smoke push:

```sh
GLCTL_COMPANY_ID=demo_company \
GLCTL_DATA_DIR="$HOME/.glctl/data" \
./target/release/glctl push --remote http://127.0.0.1:3201
```


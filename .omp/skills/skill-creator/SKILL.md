---
name: skill-creator
description: >-
  Create, refactor, validate, evaluate, and package reusable Agent Skills built
  around SKILL.md. Use when asked to create a skill, convert a repeated workflow
  into a skill, improve triggering or progressive disclosure, reorganize skill
  references/scripts/assets, validate frontmatter and links, compare skill
  behavior, or produce a portable .skill archive. Do not use for ordinary code
  changes that do not define or maintain an Agent Skill.
metadata:
  domain: skill-authoring
  audience: skill-authors
---

# Skill Creator

Create focused Agent Skills that trigger for the right requests, provide an
executable workflow, and load detailed context only when needed. Optimize for
future agent use—not for documenting the current editing session.

## Principles

1. **Description controls discovery.** State what the skill does, concrete
   requests that should trigger it, and an important boundary when neighboring
   skills could be confused.
2. **Keep `SKILL.md` procedural.** Put the common workflow and decision points
   in the main file; move conditional detail, long examples, schemas, and
   vendor-specific guidance into `references/`.
3. **Create only justified files.** Add a reference, script, asset, or test only
   when a named workflow step consumes it. Empty scaffolding increases noise.
4. **Prefer durable instructions.** Exclude raw eval output, changelog prose,
   temporary paths, secrets, and rules tailored to one example.
5. **Verify the target runtime.** Discovery paths, optional frontmatter,
   permissions, and packaging vary. Read current official docs or nearby valid
   skills instead of borrowing another agent's conventions.
6. **Test progressively.** Always validate structure and links. Add formal
   trigger or behavioral evaluation only when risk, reuse, or the user request
   justifies it.

## Workflow

### 1. Scope and inspect

Extract from the request:

- outcome the skill should enable;
- concrete requests, files, or workflows that should trigger it;
- similar requests that should not trigger it;
- target agent/runtime and skill location;
- required artifacts or response format;
- whether this is create, revise, split, merge, validate, evaluate, or package.

Ask at most one concise question only when a missing decision would materially
change the result. Otherwise make a reasonable choice and proceed.

Before creating files:

1. Locate the repository root and configured skill directories.
2. Search existing skill names/descriptions for overlap.
3. Read the closest relevant skills and repository conventions.
4. Consult current runtime documentation when fields, discovery, permissions,
   or commands matter.
5. Reuse or extend a suitable skill instead of creating a duplicate. Split only
   when responsibilities or trigger intent are genuinely distinct.

Do not overwrite unrelated work. Preserve the repository's runtime, package
manager, formatting, and validation conventions.

### 2. Design the skill

Decide these before writing:

| Decision | Requirement |
|---|---|
| Name | Stable kebab-case folder and matching frontmatter name |
| Trigger contract | Positive requests plus meaningful near-miss boundary |
| Default workflow | Smallest sequence that handles the common request |
| Decision points | Conditions that select references, scripts, or branches |
| Deliverables | Exact files or response shape produced |
| Verification | Structural, behavioral, and optional evaluation checks |
| Supporting files | Only files with a named consumer |

Preferred layout:

```text
skill-name/
├── SKILL.md
├── references/       # Optional conditional guidance
├── scripts/          # Optional deterministic helpers
├── assets/           # Optional templates/static files
└── tests/ or evals/  # Optional validation fixtures
```

Avoid reference chains. Link supporting files directly from the workflow step
that needs them and state when to load or run each one.

### 3. Write frontmatter

Use the portable minimum unless verified runtime features are needed:

```yaml
---
name: skill-name
description: >-
  What the skill does. Use when asked for concrete workflow A, artifact B, or
  follow-up C. Do not use for adjacent workflow D.
---
```

Requirements:

- `name` matches the folder and uses lowercase letters, digits, and single
  hyphens;
- `description` includes what, when, and a boundary—not marketing copy or
  implementation detail;
- include follow-up verbs such as revise, rerun, validate, migrate, or package
  when supported;
- add `license`, `compatibility`, `metadata`, or runtime-specific permissions
  only after verifying support;
- keep metadata simple and serializable.

For Goose, official docs recommend project skills at
`.agents/skills/<name>/SKILL.md` and global skills at
`~/.agents/skills/<name>/SKILL.md`, with compatibility discovery for legacy
locations. Follow the current repository's configured location unless asked to
migrate it.

### 4. Write the workflow and supporting files

A good default body is:

```markdown
# Skill Title
One paragraph stating the outcome.

## Principles
Only durable constraints that affect decisions.

## Workflow
### 1. Inspect
### 2. Plan
### 3. Execute
### 4. Verify

## Output
Strict artifact/response format, when needed.

## References
Only local supporting files and when to load each one.
```

Use direct imperative instructions. Keep common-path behavior in `SKILL.md`.
Move content to a reference when it is specific to one platform/provider,
lengthy, optional, or independently versioned. Do not duplicate a rule across
multiple files.

Add a script only for repeated deterministic parsing, validation, conversion,
packaging, or report generation. Give it explicit arguments, actionable errors,
a guarded entry point, safe/idempotent behavior where practical, and basic
tests. Do not replace agent judgment with brittle automation.

Use assets for required templates, fixtures, or static UI. Do not ship caches,
virtual environments, generated reports, private eval runs, or credentials.

### 5. Validate

Always:

- parse YAML frontmatter;
- verify required fields, name format, and folder match;
- verify optional fields against the target runtime;
- resolve every local Markdown link and referenced file;
- scan for stale paths, unsupported commands, secrets, temporary notes, and
  unrelated generated artifacts;
- dry-run one realistic request and one boundary request by following the
  written workflow exactly.

Run the bundled structural validator from `skills/skill-creator`:

```bash
uv run --with pyyaml python3 scripts/quick_validate.py /path/to/skill
```

A validator pass proves structure, not trigger quality or task success.

### 6. Evaluate only when useful

For small edits, manual dry runs are enough. For high-value or frequently reused
skills, create:

- varied should-trigger prompts;
- near-miss should-not-trigger prompts;
- behavioral tasks with objective assertions;
- a previous/baseline version when comparison is meaningful;
- a holdout set when optimizing descriptions.

Keep evaluation output outside the production package. Load
[`references/schemas.md`](./references/schemas.md) only when using the bundled
JSON evaluation/report utilities. Fix the general trigger, workflow, or output
contract behind a failure; do not overfit instructions to one prompt.

### 7. Package and report

Package only when requested. Validate first, then run from
`skills/skill-creator`:

```bash
uv run --with pyyaml python3 -m scripts.package_skill /path/to/skill /output/dir
```

Inspect the archive and verify it contains only the skill and required support
files.

Report concisely:

```markdown
## Summary
[What was created or improved]

## Changed Files
- `path` — change and reason

## Verification
- Frontmatter/structure: pass/fail
- Local references: pass/fail
- Manual dry run: pass/fail/not run
- Automated checks/evals: command and result, or not run

## Remaining Risks
[Only unresolved or version-dependent items]
```

Do not add relationship sections for unrelated orchestration systems. The skill
must remain independently understandable and reusable.

## Definition of done

- [ ] Existing skills were checked before creating a new one.
- [ ] Folder name, frontmatter name, and trigger description agree.
- [ ] `SKILL.md` contains the common workflow without optional-document bloat.
- [ ] Every reference/script/asset has a consumer; every local link resolves.
- [ ] Runtime-specific claims and commands were verified.
- [ ] Structural validation and positive/boundary dry runs pass.
- [ ] Formal evaluations were run only if requested or risk-justified.
- [ ] Portable packages exclude secrets, caches, generated output, and private
      evaluation artifacts.

## Optional local tools

| Need | File |
|---|---|
| Structural validation | `scripts/quick_validate.py` |
| Portable `.skill` archive | `scripts/package_skill.py` |
| Evaluation JSON formats | `references/schemas.md` |
| Trigger checks | `scripts/run_eval.py` |
| Description optimization loop | `scripts/run_loop.py` |
| Benchmark aggregation | `scripts/aggregate_benchmark.py` |
| Review UI | `eval-viewer/generate_review.py` |

Load or run optional tooling only when the request needs it. The core create/edit
workflow must not depend on evaluation or reporting utilities.

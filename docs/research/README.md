# Hum Research Workflow

Date: 2026-07-06

## Purpose

Research artifacts keep Hum from depending on stale chat memory, vibes, or one
person's recall. They are inputs to design, not automatic ground truth.

The durable flow is:

```text
external research snapshot -> distilled Hum doctrine -> architecture and roadmap gates -> implementation tasks and tests
```

## Directory Layout

- `YYYY-MM-DD-topic.md`: normalized research snapshot from an external research
  run.
- `prompts/*.md`: reusable prompts for future research runs.

Store raw exports outside the repo until they pass import and hygiene checks.

For interviews, videos, and transcripts, commit distilled research notes and
design consequences instead of raw transcript text unless the license and
copyright status are explicit.

## Import Rule

Use the repo import tool for Markdown exports:

```powershell
.\tools\import_research_report.ps1 -SourcePath <export.md> -Slug <topic> -Date <yyyy-mm-dd>
```

The importer:

- reads the export as strict UTF-8
- strips UI-only Deep Research citation markers
- normalizes typographic punctuation to ASCII
- rejects unsupported non-ASCII characters
- writes UTF-8 without BOM

After importing, run:

```powershell
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
```

## Prompt Requirements

Future research prompts should require:

- Markdown output
- direct durable source URLs in the body and bibliography
- clear `Fact` versus `Inference` labels
- absolute dates for time-sensitive claims
- official or primary sources before commentary
- a blocker list with severity and target milestone
- a "what to stop doing or defer" section
- a "what would change the recommendation" section

Do not rely on UI-only citation markers. If the export has citations that cannot
survive as Markdown links or URLs, keep the report as background research and
create a follow-up prompt that asks for durable citations.

## Ground Truth Rule

A research snapshot can challenge Hum's direction, but it does not change the
language by itself. Any design change still has to land in the relevant ground
truth document and answer the update questions in
[../ARCHITECTURE.md](../ARCHITECTURE.md).

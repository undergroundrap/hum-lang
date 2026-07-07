# Hum Release Manifest Schema

Date: 2026-07-07

Current schema: `hum.release_manifest.v0`

## Purpose

The release manifest is the machine-readable companion to the changelog and
per-version release notes. It records the intended version, tag name, status,
release artifacts, verification commands, and publishing state without creating
a tag or touching a remote.

The manifest should stay portable. It must not contain local absolute paths,
user names, secrets, tokens, private remotes, or machine-specific install paths.

## Current Manifest

The current release candidate manifest is:

- [releases/v0.0.1.manifest.json](releases/v0.0.1.manifest.json)

## Top-Level Shape

```json
{
  "schema": "hum.release_manifest.v0",
  "version": "0.0.1",
  "tag": "v0.0.1",
  "date": "2026-07-07",
  "status": "pre-alpha",
  "commit": {},
  "source_artifacts": {},
  "verification": [],
  "publishing": {},
  "compatibility": {}
}
```

## Fields

- `schema`: schema name, currently `hum.release_manifest.v0`
- `version`: SemVer version from `VERSION`
- `tag`: intended annotated Git tag name
- `date`: release-note date in `YYYY-MM-DD`
- `status`: release maturity, currently `pre-alpha`
- `commit`: how the final verified commit is resolved
- `source_artifacts`: release files that must exist
- `verification`: commands required before tag creation
- `publishing`: whether a tag, remote push, or package publish has happened
- `compatibility`: stability promises and non-promises

## Rules

- `version` must match `VERSION`.
- `tag` must be `v` plus `version`.
- `status` must match the release notes.
- `source_artifacts.release_notes` must point at the per-version note.
- `source_artifacts.changelog` must point at `CHANGELOG.md`.
- `verification` must include `tools/check_all.ps1`,
  `tools/check_clean_checkout.ps1`, and `tools/check_tag_readiness.ps1`.
- Before tag creation, `publishing.tag_created`,
  `publishing.remote_touched`, and `publishing.public_package_published` must
  be `false`.
- The final commit hash should come from `tools/check_tag_readiness.ps1` at tag
  time, not from a self-invalidating committed hash.

## Non-Goals For V0

V0 does not promise:

- signed provenance
- binary artifact checksums
- SBOMs
- package registry metadata
- remote publishing status

Those belong to later alpha release evidence.

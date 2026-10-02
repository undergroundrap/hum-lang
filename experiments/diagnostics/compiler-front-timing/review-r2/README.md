# Timing-observability candidate export (r2)

Review-transport bundle for the **uncommitted** production
timing-observability candidate. The candidate itself lives only in its
worktree; this directory is a byte-exact export for independent review.

- Pinned base: `a01f4dc242cd075d7aff91090280ce9c38bcc8a7` (PR #69 head; the candidate's base)
- `files/` mirrors the repository layout and holds the six candidate
  files with their exact candidate bytes.
- `SHA256SUMS` lists the SHA-256 of each file under `files/`.
- `MANIFEST.txt` lists the pinned base, the six git blob identities,
  and the package SHA-256 (SHA-256 of `SHA256SUMS`).
- `candidate.diff` is the complete unified diff of the candidate
  against the pinned base.

## Verification

From the repository root at the pinned base:

```
D=experiments/diagnostics/compiler-front-timing/review-r2
(cd $D && sha256sum -c SHA256SUMS)
for f in $(cut -d' ' -f3- $D/SHA256SUMS | sed 's|^files/||'); do
  git hash-object $D/files/$f   # compare against MANIFEST.txt
done
sha256sum $D/SHA256SUMS        # compare against package sha256 in MANIFEST.txt
```

Nothing in this directory executes in CI; no workflow, test, or
production file outside it is touched by this commit.

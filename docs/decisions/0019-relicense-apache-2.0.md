# 0019: Relicense From AGPL-3.0 To Apache-2.0 With A Trademark Policy

Date: 2026-07-12
Status: accepted by the BDFL (licensing is a reserved matter)

## Context

Hum shipped under AGPL-3.0 with an added visible-attribution term. The intent
was: let people use Hum, but stop a larger party from taking it and claiming
authorship. Investigation showed the license and the goal were mismatched.

AGPL does a different job than the one intended:

- It imposes strong network copyleft, which corporate legal teams broadly ban
  outright (Google's public policy forbids AGPL use). For a language whose
  target users include security teams, regulated industries, and companies,
  AGPL blocks the very audience the project wants, and is hostile to any
  acquisition path.
- It does not protect the project's name or authorship. Copyright licenses do
  not govern names or credit; a compliant fork could still be marketed as its
  own language.

The actual goal splits into two mechanisms that are not the license's copyleft:

1. Broad usability, including commercial and closed-source and acquisition
   friendliness: served by a permissive license.
2. Protection of the "Hum" name and authorship: served by a trademark policy,
   not by copyleft.

## Decision

Relicense the Hum toolchain and repository under the Apache License, Version
2.0, and add an explicit name/trademark policy.

1. `LICENSE` contains the verbatim canonical Apache-2.0 text.
2. `Cargo.toml` declares `license = "Apache-2.0"`.
3. `NOTICE.md` carries copyright and a courtesy attribution request under
   Apache-2.0 Section 4(d).
4. `TRADEMARK.md` states the "Hum"/"hum-lang" naming policy: the code is free
   under Apache-2.0; the name may not be used to imply official status or
   authorship of a fork.
5. All AGPL/Affero references in repository docs are removed or replaced.

Apache-2.0 rather than MIT because Apache-2.0 is the enterprise and
acquisition standard (LLVM, Swift, Kubernetes, Android), carries an explicit
patent grant (Section 3) that acquirers require, and includes a trademark
non-grant (Section 6) that pairs cleanly with the separate name policy. MIT
remains a valid future option, and dual MIT-or-Apache-2.0 could be added later
without contributor friction while the author is sole copyright holder.

## Consequences

- Anyone may use, modify, distribute, and build commercial or closed-source
  work on Hum under Apache-2.0. The adoption and acquisition path is open.
- The patent grant gives users and any future acquirer patent peace on
  contributions.
- The name is protected by policy, not by copyleft: the one thing the previous
  license could not actually do is now the thing that is actually done.
- Mandatory visible-credit-as-license-term is dropped; credit becomes a
  courtesy request. This removes adoption friction while `NOTICE.md` still
  propagates informationally under Section 4(d).

## Alternatives Rejected

- Keep AGPL-3.0: blocks the target market and the acquisition path, and never
  protected the name anyway.
- MIT alone: valid and maximally simple, but no explicit patent grant and a
  weaker enterprise/acquisition signal than Apache-2.0.
- Link to the Apache text instead of embedding it: rejected. License scanners,
  GitHub detection, and corporate legal expect full text in `LICENSE`; a
  link-only file would harm the adoption story. The canonical text was fetched
  verbatim to avoid transcription error.
- Registered trademark now: unnecessary and costs money; common-law policy in
  `TRADEMARK.md` is free and sufficient pre-adoption. Registration remains a
  later option if traction warrants it.

## BDFL Note

This is a reserved matter under GOVERNANCE.md and was decided directly by the
copyright holder, not under delegated authority. Relicensing is cheap now
because the author is the sole copyright holder; it becomes hard once outside
contributors hold copyright. Doing it before going public and before accepting
external contributions is deliberate.

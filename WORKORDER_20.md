# Hum Work Order 20: Encode And Verify Canonical Backend Input Bytes

Date: 2026-08-11
<!-- hum-active-workorder:v1 -->
Status: Unit A's implementation and publication-status chains are complete,
published, and terminal-green. The encoder-interface amendment and its
publication-status chain are also complete, published, and terminal-green.
Published `main` is
`74eb0396a19ea1a058bd3fed05939c1cda7ba5a5`.

The exact seventeen-path Unit B candidate completed its one authorized Fast
red after candidate evidence began and remains parked losslessly at stash
commit `303ee9af93696409bea66d3f8a379cb1a8cf8e1a`, tree
`1f2084dd5f5e535f8cd41a3be07b7fba6b50b8a5`, on first parent
`11e037c06d70cd822e52a58f1524ae7cd0701475`. Unit B remains stopped. The
completed-red result is permanent and receives no success credit.

The independently accepted Unit B Fast-boundary recovery amendment was
committed as `74eb0396a19ea1a058bd3fed05939c1cda7ba5a5`, with parent
`11e037c06d70cd822e52a58f1524ae7cd0701475` and subject
`docs(workorder): define unit b fast recovery`. Its sole path was
`WORKORDER_20.md`, with `+462/-51`, blob
`4c83fcf74813d33065599bc640641299b623f529`, and file SHA-256
`3eb604beb58a4065205411c19ee420248ad051b78446b07fcce506a4fb76d9e7`.
It was published by a normal non-force fast-forward of `main` only from
`11e037c06d70cd822e52a58f1524ae7cd0701475` to
`74eb0396a19ea1a058bd3fed05939c1cda7ba5a5`.

Workflow `ci`, run `31667895670`, attempt 1, tested exact SHA
`74eb0396a19ea1a058bd3fed05939c1cda7ba5a5` and concluded `success`. Ubuntu
job `94346311867` and Windows job `94346311903` independently selected
`mode=full;reason=no_status_transition`, with empty anchor and transition
fields and zero run, attempt, and job binding fields. Both completed full
preflight successfully, enforced exact selector inventory `101/101`, passed
text hygiene and public readiness for 532 files, passed alpha claims and
release readiness `0.0.1`, and emitted exactly one terminal
`All Hum preflight checks passed.` marker.

Ubuntu passed suites `450/450`, `13/13`, and `60/60`, then completed all
14,226 Exhaustive cases with seed `0x48554D5F5345414C`. Windows passed suites
`465/465`, `16/16`, and `60/60`, and correctly skipped only the duplicate
Exhaustive producer.

This status record grants no Unit B recovery authority. Candidate restoration,
either frozen correction, renewed validation or Fast, Unit B review,
implementation commit, publication, closeout, stash cleanup, Work Order
organization, semantic-coordinate research, and later backend work remain
unauthorized until their separately frozen gates are reached.

Owner: BDFL (Ocean).
Author: Work Order 20 architect-author. The author may not independently review
or accept this planning package or any implementation candidate produced under
it.

Planning baseline: `HEAD`, local `main`, cached `origin/main`, and live remote
`main` are all `74913b5fa459e51cb6dc5bd841dc1717ca7ecab4`, the published Work
Order 19 closeout. The worktree was clean, the index empty, no untracked file
existed, and ahead/behind was `0/0` before this two-document draft began.

## Closed predecessor and accepted evidence

Work Order 19 is fully closed. Its accepted implementation commit is
`811588db0bbdbd42e0637d5d50c84ef72923f214`, with subject
`feat(ir): bind backend facts to program lineage`. It contains exactly twelve
paths and `+2,319/-46`.

Required full-lane CI passed in workflow `ci`, run `31553589478`, attempt 1,
testing that exact commit. Ubuntu job `93981274365` and Windows job
`93981274258` both succeeded in full mode with
`reason=no_status_transition`. The accepted implementation binds every
pre-verifier semantic fact to one exact Program-owned canonical minimal-add
operation through private, compiler-sealed, report-bound authority.

The Work Order 19 closeout commit is
`74913b5fa459e51cb6dc5bd841dc1717ca7ecab4`, with subject
`docs(workorder): close work order 19`. Fast-lane closeout CI passed in
workflow `ci`, run `31556844526`, attempt 1. Ubuntu job `93990907165` and
Windows job `93990907201` both succeeded with
`mode=fast;reason=eligible_status_chain`.

The recovery stash remains non-authoritative parked evidence:

- commit: `73101039f5e3faf0c802d4f723add1b891c51602`;
- tree: `535198cd6c9fdbd2fb713a30266530cb47e766c0`.

All archive refs are immutable historical evidence and may not be restored,
rewritten, merged, or cherry-picked. The older recovery stash may not be
applied, popped, dropped, or used as authority. The parked Unit A stash may be
restored only through the exact amendment-publication and BDFL-resumption
lifecycle below; no other stash operation is authorized.

## Planning review history and final BDFL re-envelope

The first independent pre-issuance review found the architecture satisfiable
but required reachable mutations, honest capability-lifetime wording, exact
SHA/CLI closure, and a sustainable producer/verifier split. The bounded author
correction produced the historical fourteen/fourteen/ten/eighteen topology.

The fresh terminal corrected-document review confirmed that every original
P1/P2 was closed, then found one omitted established durable consumer:
`docs/CAPABILITIES_SCHEMA.md`. Both units intentionally change
`src/capabilities.rs` and the public `hum capabilities` schema/command
inventory, so leaving the catalog unchanged would publish documentation/code
drift. Removing capability registration was rejected as a workaround because
it would contradict the required public discovery surface.

The BDFL did not reopen the exhausted ordinary correction cycle. The BDFL
first authorized one terminal document-only envelope amendment that added the
capabilities catalog and produced the historical fifteen/fifteen/eleven/
nineteen topology. A fresh independent terminal-amendment review confirmed
that correction, then found a second established durable public command
inventory omitted from the envelope: `docs/LANGUAGE_REFERENCE.md`. Its
`Current Commands` table and bootstrap examples must change whenever these two
public routes change.

The BDFL then authorized the final pinned document-only re-envelope used for
the published planning package. That now-historical topology was sixteen Unit A
paths, sixteen Unit B paths, twelve shared paths, and twenty union paths, with
a necessary twenty-first path as the mandatory stop.
The historical eighteen- and nineteen-path packages remain accurately recorded
as the packages reviewed at those gates; this amendment does not retroactively
rewrite either review or authorize implementation. The ordinary planning
correction cycle remains consumed. This BDFL-directed Fast-boundary amendment
is a separate prospective accounting and evidence ruling, not another ordinary
author correction. Any non-`ACCEPT` result from its fresh independent review
returns Work Order 20 directly to the BDFL and grants no automatic author edit.

## Unit A Fast-boundary amendment

The first Unit A implementation was based on published `main` at
`3156dd4869b3960a85eb63c6ba906b5c2b9916c8`. Its product and focused evidence
were substantially green, but its final Fast completed red after candidate
evidence began. That result was a valid stop under the Work Order then in
force. It is not retroactively reclassified as green.

The exact candidate is preserved by Git object identity, not stash ordinal:

- stash commit: `bd6d2722cffa50da8463201204a48f4a7305ae1b`;
- stash tree: `1aabe316b01345ad2f2cd589f95b64b598305bb6`;
- first parent: `3156dd4869b3960a85eb63c6ba906b5c2b9916c8`;
- message: `wo20-unit-a-fast-boundary-stop-2026-08-12`;
- reconstructed scoped sixteen-path tree:
  `46eb384f4f36218b71525cba758d60f2881c6ba5`.

The older recovery stash remains separate and unchanged at commit
`73101039f5e3faf0c802d4f723add1b891c51602`, tree
`535198cd6c9fdbd2fb713a30266530cb47e766c0`, first parent
`0396399c94f5e43511f3811319320a6ca2db0b93`. Neither stash is current
authority, and neither may be applied, popped, dropped, renamed, recreated, or
rewritten by this amendment.

### Exact stopped-candidate evidence

The parked candidate contains exactly the accepted sixteen Unit A paths. Its
measured evidence is:

- raw diff: `+1,474/-459`;
- whitespace-insensitive diff: `+1,473/-458`;
- production Rust: 742 lines;
- permanent tests/proofs: 418 lines;
- documentation/tool/fixture/catalog/reference: 314 lines;
- authenticated Work Order 19 issuer relocation: 410 deleted source lines;
- relocated production subset: 243 lines;
- ordinary/non-relocation deletions: 49 lines;
- new production logic excluding moved production: 499 lines;
- golden artifact: 8,715 bytes;
- payload and artifact SHA-256:
  `a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f`;
- root suite: 465/465;
- both new selectors: selected one, executed one, passed one;
- exact-selector inventory: 101/101;
- the SHA-round-constant and envelope-order producer mutations both failed
  their exact selector as intended;
- final Fast: exit 1 after 298.9 seconds;
- terminal Fast-success marker: absent;
- local Exhaustive: not run; and
- nothing staged, committed, or pushed.

The stopped `tools/check_all.ps1` blob is exactly
`2d3d647821905ab2edf724b761e7cd7ffdfbe2f5`.

### Authenticated relocation ledger

The deletion exemption below is an ownership-relocation exemption, not a
general moved-code exemption. The source is the published-parent
`src/ir_readiness.rs` blob
`f62b8f5d7912989d0d716f05656e4aff5bf3c25a`. The destination is the parked
`src/backend_input.rs` blob
`e0ff799e6b2ffb14a23cc3adc5c69218f05e1b12`; the retained compatibility
bridge is in parked `src/ir_readiness.rs` blob
`163e0d38840b0caeafced5da0fe0679ad27423ca`.

Every locator below is a content identity of the form `Git blob:Lx-Ly`, with
one-based inclusive lines over the exact LF Git blob. A comma joins
non-contiguous ranges in the same blob. This is reproducible without applying
the stash and is the stable byte/content identity for each block. To keep the
ledger readable, `S` means the full source blob
`f62b8f5d7912989d0d716f05656e4aff5bf3c25a`, `D` means the full destination
blob `e0ff799e6b2ffb14a23cc3adc5c69218f05e1b12`, and `B` means the full bridge
blob `163e0d38840b0caeafced5da0fe0679ad27423ca`.

| # | Source block and identity | Class | Source lines | Destination counterpart and identity | Destination lines | Equivalence rule |
| ---: | --- | --- | ---: | --- | ---: | --- |
| 1 | export/module/import scaffold `S:L61-L65` | production | 5 | bridge `B:L61` plus explicit imports `D:L1-L10` | 11 | module unwrap and explicit imports only |
| 2 | `REQUIRED_PASSES` `S:L66-L82` | production | 17 | `D:L20-L36` | 17 | token-identical after one-level dedent |
| 3 | facts/access types `S:L83-L102` | production | 20 | `D:L57-L76` | 20 | token-identical after dedent |
| 4 | final-lineage validator impl `S:L103-L219` | production | 117 | `D:L77-L193` | 117 | token-identical after dedent and exact constant aliasing |
| 5 | facts snapshot `S:L220-L241` | test/proof | 22 | `D:L194-L215` | 22 | token-identical after dedent |
| 6 | access impl `S:L242-L248` | production | 7 | `D:L216-L222` | 7 | token-identical after dedent |
| 7 | access snapshot `S:L249-L252` | test/proof | 4 | `D:L223-L226` | 4 | token-identical after dedent |
| 8 | facts assembly `S:L253-L300` | production | 48 | `D:L227-L274` | 48 | token-identical after dedent and exact constant aliasing |
| 9 | test corruption hook `S:L301-L302` | test/proof | 2 | `D:L275-L276` | 2 | token-identical after dedent |
| 10 | `issue_assembled` `S:L303-L315` | production | 13 | `D:L277-L289` | 13 | token-identical after dedent |
| 11 | old issuer and wrapper call `S:L316-L326,L480` | production | 12 | artifact issuer and compatibility wrapper `D:L290-L326` | 37 | old profile/assemble/issue/consume path retained; 25 added lines are new Unit A logic |
| 12 | foreign-final-lineage helper `S:L327-L346` | test/proof | 20 | `D:L561-L580` | 20 | token-identical after required test-only visibility qualification |
| 13 | final-lineage thread locals `S:L347-L352` | test/proof | 6 | `D:L581-L587` | 7 | both moved cells exact; one new artifact-corruption cell counts as new insertion |
| 14 | test comparison observer `S:L353-L363` | test/proof | 11 | `D:L588-L598` | 11 | token-identical after dedent |
| 15 | normal comparison observer `S:L364-L366` | production | 3 | `D:L599-L601` | 3 | token-identical after dedent |
| 16 | corruption setter `S:L367-L371` | test/proof | 5 | `D:L602-L606` | 5 | token-identical after required test-only visibility qualification |
| 17 | corruption dispatcher `S:L372-L450` | test/proof | 79 | `D:L607-L682` | 76 | same arms, values, and calls; rustfmt removes one single-expression arm block |
| 18 | checked-empty corruption helper `S:L451-L462` | test/proof | 12 | `D:L683-L694` | 12 | token-identical after dedent |
| 19 | failure-edge corruption helper `S:L463-L468` | test/proof | 6 | `D:L695-L700` | 6 | identical assignment with a rustfmt-added trailing semicolon |
| 20 | nested-module close `S:L469` | production | 1 | file-root ownership | 0 | structural unwrap; no semantic statement deleted |

The source ledger totals exactly 410 lines: 243 production and 167
test/proof. The production total includes the non-contiguous old wrapper call
at source line 480. The 409-line nested-module deletion plus that one wrapper
call is the entire authenticated relocation allowance.

As a separate semantic proof, normalized token comparison reproduced 15 of 17
direct counterparts exactly. Normalization removes one module indentation
level, aliases the two accepted string constants to `SEMANTIC_CONTRACT` and
`TARGET_CONTEXT`, and qualifies moved test helpers from `pub(super)` to
`pub(crate)`. The two remaining direct counterparts differ only by rustfmt's
removal of braces around the `empty_unsupported` single-expression match arm
and addition of a trailing semicolon to the unit-returning edge assignment.
Blocks 1, 11, and 20 are the explicitly enumerated structural/expanded cases:
the compatibility bridge remains, every old profile/assembly/validation/
consumer statement remains in the destination call chain, and all added
artifact logic continues to count fully as insertion. No semantic statement
was deleted to claim relocation credit.

Only the source-side deletion of these exact 410 authenticated lines is exempt
from Unit A ordinary-deletion accounting. New logic, formatting-only deletion,
and any omitted, changed, split, merged, or semantically rewritten block
receive no exemption unless this exact ledger still proves equivalence. An
unused part of the 410-line relocation allowance cannot become ordinary
deletion allowance.

### Relocation-aware deletion arithmetic

The original literal Unit A ceiling was 250 raw deletions. The parked candidate
has 459 raw deletions, so that original rule correctly stopped it. This BDFL
amendment changes the accounting prospectively; it does not claim that 459
passed the old ceiling.

The amended measures are:

```text
410 exact authenticated relocation deletions
+ at most 250 ordinary/non-relocation deletions
= at most 660 Unit A raw deletions
```

The parked candidate therefore measures:

```text
459 raw deletions
- 410 exact authenticated relocation deletions
= 49 ordinary/non-relocation deletions
```

It is within both amended Unit A deletion ceilings. Relocated destination
insertions continue to count fully against the production/test/total insertion
ceilings. Ordinary deletion allowance cannot transfer to Unit B, and unused
relocation allowance cannot transfer anywhere.

### Bounded obsolete-audit correction

The stopped Fast failed at `tools/check_all.ps1:1657` with:

```text
IR contract JSON must keep V0 non-emission claim
```

That audit still required the obsolete IR-contract string:

```text
"no IR emission for source files"
```

Unit A intentionally changes the `hum_ir` layer to:

```text
status = produced-unverified
role = canonical target-independent backend-input bytes awaiting IR verification
```

Only after this amendment's complete publication/status chain and a separate
explicit candidate-resumption signal may the resumed candidate change the
already authorized `tools/check_all.ps1` path. That correction may replace
only the obsolete IR-contract JSON assertion with checks over the real
`hum ir-contract --format json` output proving:

- the `hum_ir` layer exists;
- its status is exactly `produced-unverified`;
- its role is exactly `canonical target-independent backend-input bytes
  awaiting IR verification`;
- the obsolete `no IR emission for source files` claim is absent from that
  IR-contract surface;
- existing schema, semantic-owner, Core-schema, required-pass,
  typed-failure-fact, and later-backend-boundary assertions remain active;
- Unit A still claims no verified IR capability;
- Unit A still claims no backend lowering; and
- Unit A still claims no backend-adapter input authority.

The correction may not remove or weaken an unrelated assertion. Separate
Core-lower non-emission assertions remain unchanged where they are truthful.
No IR-readiness assertion may change unless independent amendment review first
proves a conflict with the published Unit A contract; the stopped evidence
identifies only the direct IR-contract assertion above.

The corrected audit must be load-bearing. Honest Unit A IR-contract JSON must
pass; removing or changing `produced-unverified` must fail; removing or
changing the exact role must fail; restoring only the obsolete non-emission
claim must not satisfy the corrected audit; and surrounding checks must remain
active. Each mutation must reach the intended corrected assertion rather than
an unrelated compilation or routing failure. This adds no parser, general JSON
framework, detached inventory, selector, or runtime credit. Inventory remains
exactly 101/101.

### Amendment publication and candidate resumption

The gates are exact and independent:

1. this document-only amendment receives fresh independent review;
2. only an unqualified `ACCEPT` advances;
3. a separately authorized local amendment commit contains only
   `WORKORDER_20.md`;
4. separate amendment publication reaches terminal-green full CI;
5. a separate amendment publication-status commit is created;
6. separate status publication reaches terminal-green fast CI;
7. the BDFL sends a separate explicit Unit A resumption signal;
8. the candidate is restored from exact stash commit
   `bd6d2722cffa50da8463201204a48f4a7305ae1b`;
9. all sixteen parked blobs and scoped tree
   `46eb384f4f36218b71525cba758d60f2881c6ba5` are independently verified;
10. only the authorized `tools/check_all.ps1` audit correction is applied;
11. checks proportional to that correction are rerun;
12. exactly one renewed direct Fast is run;
13. no local Exhaustive is run; and
14. the corrected sixteen-path candidate is frozen for fresh independent Unit
    A review.

The original completed-red Fast remains a valid stop verdict under the old
envelope. This amendment authorizes one new Fast only after the complete
durable correction chain and exact resumption. If that renewed Fast completes
red for any other reason, there is no retry or broader repair: preserve the
candidate and return to the BDFL.

This amendment changes no Unit A product semantic, path, artifact byte,
digest, CLI behavior, schema meaning, capability/catalog/reference parity,
blocked readiness state, eleven-fact order, selector ownership, or insertion
ceiling. At that historical gate it changed no Unit B envelope, budget,
verifier/capability contract, or authorization. Unit A remains unverified.

## Unit B encoder-interface stop and BDFL re-envelope

After Unit A's implementation and publication-status chains were durable and
terminal-green, the BDFL issued an explicit Unit B implementation signal. The
implementer correctly stopped before implementation because the published
Unit B envelope was not satisfiable as written:

1. canonical verification requires the decoded declared artifact ID and
   decoded payload model to pass through the one Unit A canonical encoder;
2. that encoder is private in `src/backend_input.rs`;
3. its accepted input is private producer facts tied to Work Order 19
   authority rather than a closed non-authoritative transport model;
4. the stopped sixteen-path Unit B envelope forbade editing
   `src/backend_input.rs`; and
5. copying key order, punctuation, escaping, or any other writer logic into
   `src/ir_verify.rs` would create the expressly forbidden second encoder.

The incomplete untracked `src/ir_verify.rs` file was removed. The repository
returned to clean published `main` at
`d522ef74cdf5418fe166d303d3f2ba8e49c892b5`; no Unit B candidate, selector,
validation run, Fast allowance, commit, or push exists. This is a genuine
satisfiability defect in the historical Unit B envelope, not an implementation
rejection and not evidence that a second encoder is necessary.

The BDFL therefore makes `src/backend_input.rs` intentionally shared. Unit A
remains exactly sixteen paths. Unit B becomes exactly seventeen paths. The
shared intersection becomes exactly thirteen paths. The union remains exactly
twenty paths:

```text
16 + 17 - 13 = 20
```

The published Unit A starting blob for the newly shared path is
`e0ff799e6b2ffb14a23cc3adc5c69218f05e1b12`. Unit B must authenticate that
blob before editing and must account for the complete Unit B diff of
`src/backend_input.rs`. An eighteenth Unit B path or a twenty-first union path
is an unconditional stop.

This re-envelope changes only the private canonicalization boundary needed to
make the already-frozen verifier contract satisfiable. It does not authorize a
second writer, change any Unit A byte or public surface, add a selector, alter
the verifier/capability/report contract, or grant Unit B implementation
authority. Unit B remains paused until this amendment completes independent
review, local commit, full-CI publication, publication-status recording,
fast-CI status publication, and a new explicit BDFL resumption signal.

## Unit B completed-red Fast-boundary recovery amendment

The encoder-interface amendment completed its independent review, local
commit, full-CI publication, publication-status record, and terminal-green
fast-CI status publication. The BDFL then issued the separate explicit Unit B
resumption signal. The resulting implementation produced the exact frozen
seventeen-path candidate below.

The candidate's one direct Fast completed red after candidate evidence had
begun. This was not a launcher or environment failure. It was not a semantic-
verifier, capability, lifetime, mutation, or architecture rejection. It was a
genuine public catalog-parity candidate defect caught by the unchanged
production audit. The Work Order's completed-Fast-failure stop rule was obeyed:
there was no same-run repair or retry, and the candidate was parked losslessly.
The historical Fast remains completed-red forever and receives no success
credit. Recovery is possible only through this separately reviewed amendment
and the complete new publication/status/resumption chain below.

### Exact stopped candidate and completed-red evidence

The stopped candidate is identified by Git objects rather than by a movable
stash ordinal:

- stash reference at amendment authorship: `stash@{0}`;
- stash commit: `303ee9af93696409bea66d3f8a379cb1a8cf8e1a`;
- complete stash tree: `1f2084dd5f5e535f8cd41a3be07b7fba6b50b8a5`;
- first parent: `11e037c06d70cd822e52a58f1524ae7cd0701475`;
- index parent: `37515a768721c90206d6814666082392aaeffabe`;
- untracked parent: `2ed5701a68a7d5a0d09e9e48bd3c8a7c8150acb2`; and
- message: `On main: wo20-unit-b-fast-boundary-stop-2026-08-12`.

The prior parked Unit A candidate remains the next stash at commit
`bd6d2722cffa50da8463201204a48f4a7305ae1b`. The older WO18 recovery stash
remains third at commit `73101039f5e3faf0c802d4f723add1b891c51602`.
None may be applied, popped, dropped, reordered, renamed, or rewritten by this
amendment.

Read-only reconstruction of the stopped stash proves:

- exactly seventeen regular `100644` paths;
- raw statistics `+2,878/-166`;
- whitespace-insensitive statistics `+2,877/-165`;
- 1,798 production Rust insertions;
- 847 permanent test/compile-proof insertions;
- 233 documentation/tool/catalog/reference insertions;
- exactly two untracked components,
  `docs/HUM_IR_VERIFY_SCHEMA.md` and `src/ir_verify.rs`;
- no `WORKORDER_20.md` change; and
- no path outside the frozen Unit B envelope.

The complete stopped and projected-corrected inventory follows. Each SHA-256
is over the exact LF Git blob bytes. The corrected columns repeat every full
identity deliberately; `same` or abbreviated object names are not durable
evidence.

| Path | Stopped diff | Stopped blob | Stopped SHA-256 | Corrected diff | Corrected blob | Corrected SHA-256 |
| --- | ---: | --- | --- | ---: | --- | --- |
| `README.md` | `+4/-1` | `5dac9428b5f29321f404612cca357674ef7f0f9a` | `feabeb8d1cfc9bc24457b822aa60e9d88db969123f09cfec3ca157af02c700a6` | `+4/-1` | `5dac9428b5f29321f404612cca357674ef7f0f9a` | `feabeb8d1cfc9bc24457b822aa60e9d88db969123f09cfec3ca157af02c700a6` |
| `docs/ARCHITECTURE.md` | `+1/-1` | `6af4b71fc58d0c15682be352488d72189847ecb5` | `9f0ac1f984e9d6fd547e92cc96865ba12d907f40f996eb445111b39cb35115cd` | `+1/-1` | `6af4b71fc58d0c15682be352488d72189847ecb5` | `9f0ac1f984e9d6fd547e92cc96865ba12d907f40f996eb445111b39cb35115cd` |
| `docs/BACKEND_CONTRACT_SCHEMA.md` | `+6/-1` | `8115ef859b75d827e4525b1dc3d582d459f3f7a6` | `1b056cae66c0369f70c9604a9867b3c9bf23da8cc320213851595a8ee6f036f5` | `+6/-1` | `8115ef859b75d827e4525b1dc3d582d459f3f7a6` | `1b056cae66c0369f70c9604a9867b3c9bf23da8cc320213851595a8ee6f036f5` |
| `docs/CAPABILITIES_SCHEMA.md` | `+3/-0` | `134857fc6844259f0059a287f20d5631a3571a29` | `162e729e6575dce7717a0ba9b035606894c6a5b6dcccda9d3f2ab09fa05779ab` | `+2/-0` | `aed5764b194d203fa1f8febfd076399b06b6cafc` | `8e98b3c90326fd59b221966b02f10369ecbdfaf5008d023756ec20f01fc2f00c` |
| `docs/HUM_IR_CONTRACT_SCHEMA.md` | `+6/-4` | `8d34102dc3a2094a891edc27fbe359ec2eeb2709` | `9c24809a74d33a1592665d311629a46f649f73b4692d62d8345aa15ebb235de6` | `+6/-4` | `8d34102dc3a2094a891edc27fbe359ec2eeb2709` | `9c24809a74d33a1592665d311629a46f649f73b4692d62d8345aa15ebb235de6` |
| `docs/HUM_IR_READINESS_SCHEMA.md` | `+12/-7` | `5292f92152e2a1107baa8266f2bcb9c73e483bcc` | `c593acde13f7a26a1d9769cca7c69e919294f502a37b453225dafeff0d600e5c` | `+12/-7` | `5292f92152e2a1107baa8266f2bcb9c73e483bcc` | `c593acde13f7a26a1d9769cca7c69e919294f502a37b453225dafeff0d600e5c` |
| `docs/HUM_IR_VERIFY_SCHEMA.md` | `+102/-0` | `10843f238874624edb5b39c78889b475a92d0028` | `aaf898158516d0151dc834289539d648137dcbf9999c882a5dc2f5add20a0dd1` | `+102/-0` | `10843f238874624edb5b39c78889b475a92d0028` | `aaf898158516d0151dc834289539d648137dcbf9999c882a5dc2f5add20a0dd1` |
| `docs/LANGUAGE_REFERENCE.md` | `+5/-1` | `ceeb526f6cd627b68f040b1bcb951badce371dd6` | `2df1a6916effab6c8b7686002fb5f891e764bd730d34437cd84b458f60f6ea25` | `+5/-1` | `ceeb526f6cd627b68f040b1bcb951badce371dd6` | `2df1a6916effab6c8b7686002fb5f891e764bd730d34437cd84b458f60f6ea25` |
| `src/backend_contract.rs` | `+2/-0` | `5d101f49fc343165a3d1d330f4739bec85e30da9` | `42f26687d20e3a83fafbe290e2c3168cb6f1ad132ccc4e429296250580417091` | `+2/-0` | `5d101f49fc343165a3d1d330f4739bec85e30da9` | `42f26687d20e3a83fafbe290e2c3168cb6f1ad132ccc4e429296250580417091` |
| `src/backend_input.rs` | `+468/-95` | `5bd4a8386834d337b06eacbadc853e9370abed52` | `8ab1ea4197cb12a8f635f1dce0c1c5453187361a5b485f8b6808be6bcb1a46f1` | `+468/-95` | `5bd4a8386834d337b06eacbadc853e9370abed52` | `8ab1ea4197cb12a8f635f1dce0c1c5453187361a5b485f8b6808be6bcb1a46f1` |
| `src/capabilities.rs` | `+25/-4` | `1843bf7282ae77c375579542bab38a550555ac16` | `57e306773d89dccca6bce4987b333e81826921cca5557ac479a5159f04d6fe43` | `+25/-4` | `1843bf7282ae77c375579542bab38a550555ac16` | `57e306773d89dccca6bce4987b333e81826921cca5557ac479a5159f04d6fe43` |
| `src/ir_contract.rs` | `+9/-11` | `5113d16b8e705550812f05df6d2a011f2dcbc22c` | `5a11f8945b5124e4d28e3b288cd2d89b6a1ce6489be7ced76b164cfe692a8544` | `+9/-11` | `5113d16b8e705550812f05df6d2a011f2dcbc22c` | `5a11f8945b5124e4d28e3b288cd2d89b6a1ce6489be7ced76b164cfe692a8544` |
| `src/ir_readiness.rs` | `+232/-28` | `2996dfeb2b2a53d3ccb693bd29a45054435486a4` | `f85734ccd6d653ce6c5522fd4417871ff7f6105025db82f6578522b0ac72f6c0` | `+232/-28` | `2996dfeb2b2a53d3ccb693bd29a45054435486a4` | `f85734ccd6d653ce6c5522fd4417871ff7f6105025db82f6578522b0ac72f6c0` |
| `src/ir_verify.rs` | `+1,833/-0` | `0542b8f5e6d539612486fdad20e5f4a94c1c240d` | `985c67f4a150f4699499a9dfb5f2cb23f55f041b6b9358ed504cdd4a841c0745` | `+1,833/-0` | `0542b8f5e6d539612486fdad20e5f4a94c1c240d` | `985c67f4a150f4699499a9dfb5f2cb23f55f041b6b9358ed504cdd4a841c0745` |
| `src/main.rs` | `+70/-2` | `3558a09f3d1bfb2f971244d7f45014762ba912c3` | `456b715c522a62b84112b1202a4f4fa1457dd6db2202203f4d34a8ba17be3d93` | `+70/-2` | `3558a09f3d1bfb2f971244d7f45014762ba912c3` | `456b715c522a62b84112b1202a4f4fa1457dd6db2202203f4d34a8ba17be3d93` |
| `src/version.rs` | `+6/-1` | `4a5cef6f09da0d5b8b400cc11b97871d88339aa0` | `e056d5283b5415d72edeb7f56af227175dca8793ccefa16a2c98e8fe0b7b28c1` | `+6/-1` | `4a5cef6f09da0d5b8b400cc11b97871d88339aa0` | `e056d5283b5415d72edeb7f56af227175dca8793ccefa16a2c98e8fe0b7b28c1` |
| `tools/check_all.ps1` | `+94/-10` | `d6792a184ff9635d2b502971a9f6bc908ba42635` | `b36bad081cab8d5131d5c92c602001959dc9c3701b539815176eb16813237fa3` | `+159/-18` | `d1ebd59b4d93221067be2fe04014e08855fe481c` | `f0d34b270215d5b5ab8eba284140d9c841ccc59729d786dbb4f7d492f3e9a9cd` |

The stopped projection manifest is the 2,299-byte UTF-8 sequence formed by
sorting the table by ordinal path bytes and writing one line per path as
`100644<TAB><blob><TAB><sha256><TAB><path><LF>`. Its SHA-256 is
`ecdb56f240892c8598da1b22ed4bfd8de9c3ff82a60049693565e25ed742f2f0`.
This makes the combined identity reproducible without applying the stash.

The completed-red Fast evidence is exact:

- exit: 1;
- duration: 188.133 seconds;
- candidate evidence had begun;
- root suite: 468/468;
- all five Work Order 20 selectors were each uniquely selected and passed
  1/1/0;
- failure: `Work Order 20 capability catalog entries drifted`;
- failing production audit: `tools/check_all.ps1:446`;
- defect: `docs/CAPABILITIES_SCHEMA.md` contained the exact line
  ``- `hum backend-input <file>` `` twice, reported at candidate lines 114 and
  128;
- terminal `All Hum preflight checks passed.` markers: zero;
- exact 104/104 selector inventory was not reached; and
- no retry, repair, local Exhaustive, commit, or push occurred.

### Independent recovery-amendment finding and bounded closure

The first independent review of this recovery amendment accepted the proposed
catalog bytes but found the evidence plan unsatisfiable while the stopped
`tools/check_all.ps1` remained byte-identical. The stopped tool blob
`d6792a184ff9635d2b502971a9f6bc908ba42635` counts the two catalog schema and
command entries, but it does not inspect the catalog's command order. Moving
the ir-verify command therefore passes. Changing the backend-input exact-one
predicate from `-ne 1` to duplicate-tolerant `-lt 1` also passes because no
permanent adversarial harness observes the admitted duplicate. No other
selector, script, or permanent audit owns either failure.

Recovery is therefore re-enveloped as exactly two bounded changes inside the
existing seventeen-path Unit B envelope:

1. `docs/CAPABILITIES_SCHEMA.md`; and
2. `tools/check_all.ps1`.

This adds no repository path, selector, script, or framework. Unit B remains
seventeen paths, the Work Order 20 union remains twenty paths, an eighteenth
Unit B path remains a mandatory stop, a twenty-first union path remains a
mandatory stop, and the final selector inventory remains 104/104.

The fresh independent review of the first two-file projection then found one
integration defect. The shared helper was sound in isolation, but the
integrated mutation setup at projected tool lines 491-496 constructed every
variant with `$Catalog.Replace(...)`. The surrounding real production block
bound the real document only as `$Wo20CapabilityDoc`; `$Catalog` was therefore
null/unbound. With `$ErrorActionPreference = 'Stop'`, mutation construction
terminated with `You cannot call a method on a null-valued expression.` before
any variant reached the helper. That rejected projection had blob
`d64134cb7cd561bd31f5c29380a699e723c72f2c`, SHA-256
`063e2ff4244fc9f64b82ab7cd4952df19583561188b9c42e60d923de073cfdda`,
4,230 lines, and 454,486 bytes. It is historical failed review evidence, not a
live recovery identity.

This bounded correction changes only those six mutation-construction receivers
from `$Catalog` to `$Wo20CapabilityDoc`. Every variant is consequently derived
from the same real catalog text used by the honest invocation. No second source,
validator, or path is introduced, and no semantic predicate changes.

### Frozen two-file correction and projected candidate

The published Unit A occurrence of
``- `hum backend-input <file>` `` is authoritative and remains in its published
relative position. The catalog half of the future two-file correction changes
only `docs/CAPABILITIES_SCHEMA.md` from the stopped candidate. It removes the
extra Unit B-added backend-input occurrence, moves no published Unit A
occurrence, and places the sole Unit B ir-verify occurrence immediately after
the preserved backend-input occurrence. The final contiguous command order is:

1. `hum ir-readiness --format json`
2. `hum backend-input <file>`
3. `hum ir-verify [--format json] <backend-input-file>`
4. `hum core-contract --format json`

Every other catalog entry remains byte-identical and in its existing relative
order. The corrected tool audit requires exact-one parity; it may not be
weakened, bypassed, replaced with a tolerant count, or compensated by duplicate
text elsewhere. If physical line numbers shift, exact content and the frozen
relative order above control.

The projected corrected `docs/CAPABILITIES_SCHEMA.md` identity, calculated
from the stopped blob without changing the real worktree, is:

- diff against published Unit A: `+2/-0`;
- lines/LF: 178;
- bytes: 5,630;
- Git blob: `aed5764b194d203fa1f8febfd076399b06b6cafc`;
- SHA-256: `8e98b3c90326fd59b221966b02f10369ecbdfaf5008d023756ec20f01fc2f00c`;
- CR count: zero; and
- final LF: present.

The future `tools/check_all.ps1` correction starts from the stopped blob
`d6792a184ff9635d2b502971a9f6bc908ba42635` and changes only its existing WO20
capability/catalog parity region. It replaces the stopped lines 428-446 with
one focused `Test-Wo20CapabilityCatalogParity` helper, one honest invocation
over the real human and JSON `hum capabilities` results plus the real catalog,
and one in-memory six-variant rejection harness. The helper preserves every
existing production schema/command assertion and additionally owns:

- exact-one backend-input and ir-verify schema keys and their production order;
- exact-one backend-input and ir-verify command records, production order,
  command strings, schemas, and `adapter-ready` status;
- exact-one human schema and command lines plus their production order;
- exact-one catalog schema and command lines;
- the exact catalog schema boundary
  `ir_readiness`/`backend_input`/`ir_verify`/`core_contract`; and
- the exact four-command boundary frozen above.

The projected helper and harness use no selector, new script, generic document
validator, or second tolerant path. All bytes before the stopped line 428 and
after the stopped line 446 remain byte-identical; only unavoidable local
formatting inside this bounded parity region changes.

Windows PowerShell 5.1 parsed the corrected externally projected complete
script. The actual integrated region ran under stop-on-error
(`$ErrorActionPreference = 'Stop'`) with production-shaped values bound as
`$Wo20CapabilitiesHuman`, `$Wo20CapabilitiesJson`, and
`$Wo20CapabilityDoc`. The honest invocation completed successfully. Each of
the six variants was then constructed from `$Wo20CapabilityDoc`, reached the
same helper, and was rejected: the four missing/duplicate variants by the
`capability catalog entries drifted` assertion, and both order-only variants
by the `capability catalog ordering drifted` assertion. No unbound variable,
null expression, missing helper, or ignored error occurred.

Changing only `$BackendCatalogCommandCount -ne 1` to `-lt 1` in the complete
corrected projection still parsed and accepted honest input. It admitted the
duplicate-backend-input variant, after which the permanent harness failed
precisely with `Work Order 20 capability catalog mutation 0 was accepted`.
The order-only variants retained exactly one relevant occurrence and continued
to fail only the frozen contiguous-order assertion. Integrated evidence, not
helper-only evidence, is mandatory. The exact corrected projected identity is:

- diff against published Unit A: `+159/-18`;
- diff against the stopped tool: `+70/-13`;
- whitespace-insensitive diff against published Unit A: `+159/-18`;
- lines/LF: 4,230;
- bytes: 454,546;
- Git blob: `d1ebd59b4d93221067be2fe04014e08855fe481c`;
- SHA-256: `f0d34b270215d5b5ab8eba284140d9c841ccc59729d786dbb4f7d492f3e9a9cd`;
- CR count: zero; and
- final LF: present.

The complete projected corrected candidate has:

- exactly seventeen regular `100644` paths;
- raw statistics `+2,942/-174`;
- whitespace-insensitive statistics `+2,941/-173`;
- 1,798 production Rust insertions;
- 847 permanent test/compile-proof insertions;
- 297 documentation/tool/catalog/reference insertions;
- total insertions 2,942 and raw deletions 174;
- the same two candidate-added files,
  `docs/HUM_IR_VERIFY_SCHEMA.md` and `src/ir_verify.rs`; and
- no changed byte outside `docs/CAPABILITIES_SCHEMA.md` and
`tools/check_all.ps1` relative to the stopped candidate.

The corrected projection manifest uses the same sorted 2,299-byte format as
the stopped manifest, with the corrected catalog and tool identities from the
table. Its SHA-256 is
`1ad6e4f1e1909f0bd0e68c33fd1ab4fc50dbc3126a04432a52bf1634ed082bf0`.
The complete table and this manifest are the reproducible combined identity;
no unrecorded temporary tree or shell-formatted diff hash is authoritative.
The earlier one-file projection (`+2,877/-166`, whitespace-insensitive
`+2,876/-165`, 232 documentation/tool/catalog/reference insertions, manifest
`22fc089bc04e228048fa4ae86eed1b07416e4c07ff9dabdda8ed181837583060`) is a
superseded historical projection and is not an acceptable recovery identity.
The rejected first two-file projection happened to reproduce the current
numeric totals, but its manifest
`3ce498d6f279097a8ba6a6abc27307383bd73ca2cbe17f27aed4656f5a48542c`
contains the unbound-variable tool blob and is likewise historical only.

The corrected candidate remains within every Unit B ceiling:

```text
17 paths                         = 17
production Rust insertions       = 1,798 <= 1,800
test/compile-proof insertions    =   847 <= 1,850
documentation/tool insertions    =   297 <=   650
total insertions                 = 2,942 <= 4,300
raw deletions                    =   174 <=   550
Unit B selectors                 =     3
final selector inventory         = 104/104
```

An eighteenth Unit B path or a twenty-first Work Order 20 union path remains a
mandatory stop. Unit A's 101 published selector credits and every published
Unit A byte remain mandatory.

The two corrections change no production Rust, verifier behavior, shared model
or emitter architecture, `ComputeFromPayload`, `PreserveDeclared(decoded_id)`,
canonical equality, digest ownership, capability construction, readiness,
CLI behavior, schema other than the already frozen candidate bytes, selector,
README, language reference, architecture text, or other candidate byte. The
tool correction adds only the bounded parity helper and evidence described
above; every non-WO20 tooling behavior remains unchanged.

### Permanent catalog recovery evidence

The corrected existing Fast capability/catalog parity block and existing Unit
B selectors own the following evidence without a sixth selector or duplicate
runtime credit. Exactly one helper is the catalog validation boundary. It is
invoked exactly once on the honest real human/JSON/catalog inputs and once per
in-memory adversarial catalog. Source/topology audit must prove that complete
shape, the absence of a second tolerant path, no compensating duplicate, no
command-order bypass, no suppression of an adversarial rejection, and zero
unbound-variable or null-expression failures:

1. Honest corrected catalog: exactly one backend-input command, exactly one
   ir-verify command, exact adjacency/order at the frozen ir-readiness/core-
   contract boundary, exact schema-entry parity, and production human/JSON
   capability parity.
2. Duplicate backend-input mutation: reintroducing the removed duplicate away
   from the otherwise-correct four-command boundary must fail exact-one
   enforcement in the shared helper.
3. Missing backend-input mutation: deleting the preserved published Unit A
   occurrence must fail.
4. Duplicate ir-verify mutation: adding a second ir-verify occurrence must
   fail.
5. Missing ir-verify mutation: deleting the sole ir-verify occurrence must
   fail.
6. Two separately represented reordered-command variants: one swaps only
   backend-input and ir-verify while keeping both counts exactly one; the other
   moves the intact pair away from the frozen ir-readiness/core-contract
   boundary. Each must fail only the shared document-order check while schema
   and production capability inputs remain honest.
7. Audit-weakening mutation: in a disposable projected copy, changing only the
   helper's backend-input catalog-command comparison from `-ne 1` to `-lt 1`
   must parse and continue to accept the honest catalog. The independently
   projected weakened helper did so, then admitted the duplicate-backend-input
   variant; the permanent mutation harness detected that admission and failed
   the owning parity block.

Each mutation must reach its intended existing production parity boundary. A
stale expected string, unrelated compilation failure, alternate duplicate, or
preselected failure result earns no evidence. Syntax, missing symbols, an
unrelated assertion, or weakening detected only by source text also earns no
credit.

### Mandatory recovery lifecycle

The recovery gates are separate and exact:

1. this corrected document-only amendment is authored;
2. one fresh independent corrected-amendment review occurs;
3. only unqualified `ACCEPT` advances;
4. a separately authorized local commit contains only `WORKORDER_20.md`;
5. separate amendment publication receives terminal-green full CI;
6. a separate publication-status commit records that publication;
7. separate status publication receives terminal-green fast CI;
8. the BDFL issues a new explicit Unit B recovery signal;
9. the implementer restores exactly stash commit
   `303ee9af93696409bea66d3f8a379cb1a8cf8e1a` without consuming, dropping, or
   reordering any stash;
10. all seventeen stopped-candidate blobs and both candidate-added files are
    authenticated against the table above;
11. the implementer changes exactly `docs/CAPABILITIES_SCHEMA.md` and
    `tools/check_all.ps1` from the stopped candidate, applying both frozen
    projected identities and no other candidate byte;
12. the integrated focused honest validation, all six in-memory document
    variants constructed from `$Wo20CapabilityDoc`, the source/topology audit,
    and the independently load-bearing audit-weakening mutation all pass
    through the shared parity boundary with zero unbound/null failures;
13. all proportional checks pass;
14. exactly one renewed direct Fast runs on the corrected frozen candidate;
15. if that Fast completes red for any other reason, no repair or retry occurs;
    the candidate is preserved and control returns to the BDFL;
16. if Fast is terminal-green, the exact corrected seventeen-path candidate is
    left unstaged for one fresh complete independent Unit B review;
17. the reviewer repeats the complete Work Order 20 Unit B review, with at most
    one independently authorized reviewer Fast; and
18. only unqualified Unit B `ACCEPT` may advance to a separately authorized
    local implementation commit.

The renewed Fast does not erase, replace, rerun, or grant success credit to the
historical completed-red Fast. A non-`ACCEPT` amendment or implementation
review returns directly to the BDFL and authorizes no automatic correction.
Unit B implementation publication, status, closeout, stash cleanup, Work Order
organization, semantic-coordinate research, and later backend work remain
separately gated.

## Mandatory sufficiency ruling

The accepted Work Order 19 authority is sufficient to produce the minimum
honest `hum.backend_input.v0` artifact for the exact
`examples/core/minimal_add.hum` operation. No prerequisite Work Order is
required.

This ruling is narrow. It does not say the current compiler can encode arbitrary
tasks, multiple functions, general control flow, user-defined types, external
effects, nonempty ownership or resource state, or a platform ABI. V0 accepts
exactly one canonical `Int + Int -> Int` task with the Work Order 19 authority
chain and explicit `allocates: nothing` declaration.

The encoder may not rediscover a semantic fact from source text, public JSON,
display names, spans alone, independent checker reports, or caller-supplied
IDs. It consumes the private Work Order 19 access after the final Program-
lineage check. Process-local Program addresses remain validation authority and
must never enter persisted bytes.

### Complete fact provenance

| Required artifact fact | Classification | Authoritative source or frozen assignment |
| --- | --- | --- |
| compiler version | directly borrowed | Work Order 19 `compiler_version`, equal to `version::HUM_VERSION` |
| target-independent IR schema | fixed accepted contract | `hum.ir_contract.v0` |
| backend-input schema | fixed accepted contract | `hum.backend_input.v0` |
| feature set | fixed accepted contract | exactly `canonical_minimal_add_checked_i64_v0` |
| semantic contract | directly borrowed | `hum.canonical_minimal_add_backend_facts.v0` |
| target context | directly borrowed | `target_independent_checked_i64_v0` |
| source revision bytes | directly borrowed | parser-owned `CanonicalCoreFileBinding::source_revision` retained by Work Order 19 facts |
| source revision digest | deterministically assigned | production SHA-256 over the authenticated exact source-revision bytes; for the accepted 121-byte fixture this is `aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6` |
| semantic file ordinal | directly borrowed | parser-owned `semantic_file_index` |
| normalized path | directly borrowed | parser-owned normalized slash path |
| module identity | directly borrowed | authenticated source module `examples.core.minimal_add` |
| ordered module files | deterministically assigned | the sole authenticated file at semantic ordinal zero; V0 rejects any second file |
| function/source-item identity | directly borrowed | resolver semantic item identity and authenticated task signature |
| display name and item kind | directly borrowed | authenticated task and owner |
| linkage identity | deterministically assigned | internal-only `hum_fn_0` from the sole function ordinal; never derived from display text |
| calling convention | fixed accepted contract | target-independent `hum_internal_v0`, not a platform ABI |
| function, block, and operation IDs | deterministically assigned | closed ordinal algorithm below over authenticated order |
| section, statement, and operation source identity | directly borrowed plus deterministic assignment | authenticated owner `does` section slot, parser statement ID, Core root/value identity; V0 assigns the section-table ID from that authenticated slot and ordinal |
| ordered parameter values | directly borrowed | Work Order 19 operand value IDs in left/right parameter order |
| parameter and result types | directly borrowed plus fixed layout | checked builtin `Int`; accepted contract maps it to signed 64-bit `type:int64` |
| result value | directly borrowed | verified Core result value ID |
| canonical expression identity | directly borrowed | producer-owned canonical root node ID and result value ID |
| checked-add operator | directly borrowed | canonical binary `ParsedBinaryOperator::Add`, fixed artifact discriminant `checked_add` |
| ordered child node/value IDs | directly borrowed | two authenticated operands in canonical child positions zero and one |
| use-to-definition bindings | directly borrowed | resolver-owned canonical use node IDs, definition IDs, and semantic definition IDs |
| effect and external authority emptiness | directly borrowed | Work Order 19 distinct checked-empty `effects` and `external_authority` conclusions |
| moves, borrows, aliases, and transfers emptiness | fixed projection of direct authority | compiler-sealed `VerifiedMinimalAddOwnership` exact no-transfer conclusion; V0 emits four separately named empty arrays |
| allocation/resource emptiness | directly borrowed | compiler-sealed resource authority, one `allocates: nothing`, and checked-empty `allocations` |
| contract predicate and evidence emptiness | directly borrowed | distinct checked-empty `contract_predicates` and `evidence_obligations` conclusions |
| accepted profile | directly borrowed | compiler-sealed default `normal` profile authority |
| signed overflow edge | directly borrowed | one `signed_64/checked_add/runtime_trap_on_overflow` edge |
| exact required passes | directly borrowed | fourteen ordered, successful, one-selected, same-Program conclusions |
| unsupported or weakened state | directly borrowed | distinct checked-empty `unsupported_or_weakened` conclusion; encoded as an explicit empty table |
| diagnostic source provenance | directly borrowed | normalized source path plus authenticated task, statement, root, and operand spans/IDs |

The deterministic projections above do not invent source meaning. They assign
closed artifact-table identities, fixed V0 layout spellings, and a source digest
to facts already authenticated by the live compiler chain.

Logical Program context in persisted bytes is the source-revision digest plus
the ordered source/module/item/section/statement/operation identities. The
process-local Program address is used only while issuing the artifact and is
never serialized. Two separately parsed Programs with byte-identical source and
identical authenticated logical identities are intentionally the same portable
artifact context; "foreign Program" rejection means a mixed or substituted
logical context, not an impossible comparison with a persisted pointer.

If implementation discovers that any table field cannot be populated from one
of these exact sources without parsing rendered output or joining an independent
report, stop. Do not add the missing fact from a public projection.

## Two-unit exact result

Work Order 20 contains two separately accepted and published implementation
units within one frozen twenty-path union. The dependency order is:

```text
accepted WO19 backend facts
  -> deterministic canonical payload bytes
  -> SHA-256(payload bytes)
  -> canonical hum.backend_input.v0 envelope bytes
  -> published unverified producer boundary                 [Unit A]
  -> ir_verify(exact envelope bytes)
  -> opaque VerifiedBackendInput<'artifact>
  -> authenticated borrowed projections
  -> completed-verification IR readiness                    [Unit B]
```

Unit A is the authenticated canonical producer. It ends after the exact
minimal-add envelope is emitted and compared with the reviewed golden artifact.
Its bytes are explicitly unverified data. It adds no `VerifiedBackendInput`, no
`ir-verify`, and no readiness authority. The existing exact candidate remains
`ready_for_ir=0` with `ir_verify_not_implemented` as its sole blocker.

Unit B starts only from the accepted, published, status-recorded Unit A bytes.
It adds the strict raw-byte verifier, owned report, callback-scoped capability,
`ir-verify`, and the narrow completed-verification readiness transition. A
verifier therefore cannot target ghost bytes, while the intermediate producer
does not falsely claim verification.

The Work Order ends after one exact artifact is encoded, verified, and lent
through the opaque capability during the IR-readiness callback. Neither unit
adds a backend adapter or lowers an operation to Cranelift, LLVM, Wasm, C,
custom IR, object code, or machine code.

## SHA-256 production decision

Work Order 20 uses a narrowly scoped in-repository SHA-256 implementation in
`src/sha256.rs`. No third-party crate or build dependency is introduced.

This preserves the bootstrap rule that the compiler has no third-party crates,
keeps offline builds unchanged, and avoids adding Cargo, lockfile, license,
NOTICE, vendoring, build-script, or dependency-audit authority to this already
security-sensitive unit. The implementation is intentionally one-shot and
private to compiler artifact identity. It is not exposed as a Hum crypto API,
MAC, signature, password hash, authenticity proof, or general-purpose digest
framework.

Required implementation properties:

- pure safe Rust under the crate-wide `#![forbid(unsafe_code)]`;
- exact FIPS 180-4 SHA-256 padding, compression, and big-endian length rules;
- exact byte-length conversion using `u64::try_from(input.len())` followed by
  `checked_mul(8)` for the SHA bit length;
- a private fail-closed oversized-input error if either conversion fails, with
  no wrapping, truncation, unsafe code, or panic-dependent contract;
- lowercase 64-hex output only at the artifact boundary;
- no platform intrinsics, host-specific acceleration, locale, or environment
  input;
- no streaming or generic digest abstraction in V0; and
- one implementation site with no duplicated tool-only algorithm.

Permanent known-answer evidence includes:

- empty input;
- `abc`;
- a short input;
- 55-, 56-, 63-, 64-, and 65-byte padding boundaries;
- a multi-block standard vector;
- one million `a` bytes; and
- the exact minimal-add payload and source-revision bytes.

The production result must also compare byte-for-byte against an independent
trusted implementation in `tools/check_all.ps1`. The reference is
`System.Security.Cryptography.SHA256.Create().ComputeHash(...)`, which is
available in the PowerShell/.NET environment on both required CI platforms.
The reference implementation is evidence only and is not a production
dependency.

Any request for a crate, vendored source, build script, platform intrinsic, or
second digest path is outside the envelope and stops Unit A.

## Canonical byte grammar

### Envelope bytes

The artifact is UTF-8 without BOM. The exact envelope is:

```text
{"schema":"hum.backend_input.v0","artifact_id":"sha256:<64-lowercase-hex>","payload":<PAYLOAD>}\n
```

Rules:

- there is no leading whitespace;
- the envelope key order is exactly `schema`, `artifact_id`, `payload`;
- separators are exactly `:` and `,`, with no surrounding whitespace;
- the payload is embedded as its exact canonical object bytes;
- there is exactly one final LF byte (`0x0a`) after the closing envelope brace;
- there is no CR and no other trailing byte; and
- `artifact_id` is SHA-256 over `<PAYLOAD>` only, excluding the envelope,
  digest text, closing envelope brace, and final LF.

### Payload top-level order

The payload is one compact JSON object with no leading/trailing whitespace and
no final newline. Its keys occur exactly once in this order:

1. `compiler`
2. `source_revision`
3. `module`
4. `functions`
5. `types`
6. `definitions`
7. `effects`
8. `resources`
9. `failure_edges`
10. `unsupported`

No other top-level key is accepted.

### Exact V0 payload shape

The following is the normative shape. Angle-bracket tokens are values borrowed
or deterministically assigned under this Work Order; they are not literal
transport syntax.

```json
{"compiler":{"version":"0.0.1","ir_schema":"hum.ir_contract.v0","semantic_contract":"hum.canonical_minimal_add_backend_facts.v0","feature_set":["canonical_minimal_add_checked_i64_v0"],"target_context":"target_independent_checked_i64_v0"},"source_revision":{"id":"source:0","sha256":"sha256:<source-revision-sha256>","file_ordinal":0,"normalized_path":"examples/core/minimal_add.hum"},"module":{"id":"module:examples.core.minimal_add","name":"examples.core.minimal_add","files":["source:0"]},"functions":[{"id":"function:0","source_item_id":"<resolver-semantic-item-id>","display_name":"add","item_kind":"task","linkage":{"kind":"internal","symbol":"hum_fn_0"},"source_span":{"source_id":"source:0","line":3,"column":1},"abi":{"calling_convention":"hum_internal_v0","parameters":["<left-value-id>","<right-value-id>"],"parameter_types":["type:int64","type:int64"],"result":"<result-value-id>","result_type":"type:int64","integer_width":64,"trap_convention":"hum_checked_trap_v0"},"blocks":[{"id":"block:function:0:0","operations":[{"id":"operation:function:0:block:0:0","section_id":"section:function:0:does:0","kind":"return","statement_id":"<statement-id>","expression_id":"<root-node-id>","result_value_id":"<result-value-id>","source_span":{"source_id":"source:0","line":8,"column":5}}]}],"expressions":[{"id":"<root-node-id>","kind":"binary","operator":"checked_add","children":[{"ordinal":0,"node_id":"<left-node-id>","value_id":"<left-value-id>","definition_id":"<left-definition-id>"},{"ordinal":1,"node_id":"<right-node-id>","value_id":"<right-value-id>","definition_id":"<right-definition-id>"}],"result_value_id":"<result-value-id>","checked_type_id":"type:int64","effect_id":"effect:function:0:0","resource_id":"resource:function:0:0","failure_edge_id":"failure-edge:function:0:0","unsupported":[],"source_provenance":{"source_id":"source:0","statement_id":"<statement-id>","line":8,"column":12}}],"required_passes":[{"name":"parse","status":"passed","selected":1,"ordinal":0},"<remaining-thirteen-ordered-pass-records>"]}],"types":[{"id":"type:int64","source_type_id":"hum-type:builtin:Int","name":"Int","kind":"integer","signed":true,"bits":64}],"definitions":[{"id":"<left-definition-id>","semantic_id":"<left-semantic-definition-id>","kind":"parameter","ordinal":0,"value_id":"<left-value-id>","type_id":"type:int64","source_span":{"source_id":"source:0","line":3,"column":10}},{"id":"<right-definition-id>","semantic_id":"<right-semantic-definition-id>","kind":"parameter","ordinal":1,"value_id":"<right-value-id>","type_id":"type:int64","source_span":{"source_id":"source:0","line":3,"column":18}}],"effects":[{"id":"effect:function:0:0","effects":[],"external_authority":[]}],"resources":[{"id":"resource:function:0:0","allocation_declaration":"nothing","allocations":[],"moves":[],"borrows":[],"aliases":[],"ownership_transfers":[],"contract_predicates":[],"evidence_obligations":[],"profile":"normal"}],"failure_edges":[{"id":"failure-edge:function:0:0","value_type":"signed_64","operation":"checked_add","behavior":"runtime_trap_on_overflow"}],"unsupported":[]}
```

The line numbers shown above are derived from the accepted fixture after the
`allocates: nothing` block. The encoder must use authenticated spans and tests
must pin the actual values; no source-text search may fill them.

For the accepted fixture, `<source-revision-sha256>` is exactly
`aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6`.
The task span is line 3, column 1; the left and right parameter spans begin at
line 3, columns 10 and 18; and the return statement and expression begin at
line 8, columns 5 and 12. These values are evidence expectations, while the
production encoder still obtains them through authenticated spans.

The fourteen `required_passes` records are, in ordinal order:

1. `parse`
2. `semantic_graph_build`
3. `resolve`
4. `body_grammar`
5. `core_preview`
6. `core_lowering`
7. `core_verify`
8. `type_check`
9. `full_type_check`
10. `effect_check`
11. `ownership_alias_check`
12. `allocation_resource_check`
13. `contract_evidence_linking_checked_empty_for_exact_item`
14. `profile_check`

Each record has `status="passed"`, `selected=1`, and its zero-based ordinal.
`ir_verify` is not part of the payload's prerequisite pass array; it verifies
the completed artifact.

### Deterministic artifact IDs

V0 assigns table IDs only after all semantic inputs have passed Work Order 19:

- source ID: `source:<semantic-file-ordinal>`;
- module ID: `module:<authenticated-module-identity>`;
- function ID: `function:<function-ordinal>`;
- section ID: `section:<function-id>:<authenticated-section-name>:<section-ordinal>`;
- block ID: `block:<function-id>:<block-ordinal>`;
- operation ID: `operation:<function-id>:<block-id-suffix>:<operation-ordinal>`;
- effect ID: `effect:<function-id>:<expression-ordinal>`;
- resource ID: `resource:<function-id>:<expression-ordinal>`; and
- failure-edge ID: `failure-edge:<function-id>:<expression-ordinal>`.

For the closed V0 subset, semantic file, function, `does` section, block,
operation, and expression ordinals must each be zero. The encoder rejects
additional files, relevant sections, functions, blocks, operations, or
expressions instead of silently omitting them. Resolver definition IDs,
semantic definition IDs, parser node IDs, and Core value IDs remain producer-
owned strings and are not renumbered.

Function order is semantic file ordinal followed by authenticated recursive
item traversal ordinal. Block and operation order is the canonical Core order.
No ID or ordering may depend on a pointer address, display-name sort, hash-map
iteration, locale, OS path spelling, process randomness, or caller input.

### JSON scalar and collection rules

- strings use `\"`, `\\`, `\b`, `\t`, `\n`, `\f`, and `\r` for those exact
  characters;
- any other U+0000 through U+001F code point uses lowercase `\u00xx`;
- `/` is never escaped;
- non-ASCII Unicode scalar values are emitted as their shortest UTF-8 bytes,
  not `\u` escapes;
- lone surrogate escapes are invalid;
- integers are base-ten ASCII with no `+`, no leading zero except `0`, and no
  negative zero;
- floating-point numbers are forbidden;
- booleans are exactly `true` and `false`;
- `null` is forbidden; required empty facts use explicit empty arrays;
- every object has the schema-defined key order above and in the normative
  shape;
- arrays preserve their specified semantic order; and
- duplicate, missing, unknown, extra, or reordered keys and records are
  invalid.

Paths use `/`, never `\`, and must equal the parser-owned normalized path.
Drive letters, current directories, absolute host paths, case folding, Unicode
normalization, and filesystem canonicalization do not enter V0 bytes.

"Closed grammar" means a bounded JSON syntax followed by exact canonical-byte
enforcement; it does not mean that every syntactically legal noncanonical
spelling is rejected by the syntax reader. The verifier stages are exactly:

1. validate UTF-8/framing and decode bounded JSON syntax while retaining member
   order, every raw member/value byte span, exact payload start/end offsets,
   original string/number spellings, and duplicate-key occurrences;
2. reject duplicate keys from that ordered occurrence stream before conversion
   into any map, lookup table, or ordinary decoded model that could erase them;
3. build a private unverified model and perform closed structural, semantic,
   cross-table, and raw-payload digest validation in the prescribed order;
4. pass the decoded declared `artifact_id` and decoded payload model to the one
   repository-owned canonical encoder, which emits the canonical envelope
   spelling without recalculating or replacing that declared ID; and
5. require byte-for-byte identity between those re-encoded envelope bytes and
   the original input, including the single final LF.

The production producer uses the same encoder but supplies the digest it just
computed from canonical payload bytes. This is one encoder with two
authenticated call sites, not a second canonicalization algorithm. A generic
JSON map, generic serializer, or parse-and-normalize shortcut is forbidden.

Two selected noncanonical inputs are syntactically accepted through stage 3:

- insert exactly one ASCII space after the comma between the payload's
  `compiler` and `source_revision` members; and
- replace exactly one `/` in the `normalized_path` spelling with JSON's
  semantically equivalent `\/` escape.

For each, the harness hashes the mutated raw payload bytes and installs the
matching lowercase digest in `artifact_id`. Structure and semantics therefore
pass, and canonical re-encoding equality is the sole rejection. Other legal
JSON whitespace/escape variations remain noncanonical and may be rejected at
stage 5; malformed JSON, invalid escapes, duplicates, and closed-shape failures
retain their own earlier classes.

## Artifact production API

`src/backend_input.rs` owns the moved Work Order 19 backend-facts issuer, the
closed V0 artifact model, deterministic assignments, and the sole canonical
payload/envelope byte emitter. This removes a conceptual
`ir_readiness -> encoder -> ir_readiness` cycle while keeping producer and
verifier authority acyclic.

The published Unit A implementation currently assembles and writes directly
from private `CanonicalMinimalAddBackendFacts`. Unit B may refactor that one
file only enough to interpose one closed, explicitly unverified
`hum.backend_input.v0` model shared by the producer and verifier. The model is
owned by `src/backend_input.rs` and represents only the exact frozen V0 fields.
It contains no `Program` pointer, Work Order 19 profile/access value, lineage
permit, verified range, capability, report, generic JSON value, or authority.

The Rust-satisfiable construction boundary is an opaque crate-private
`UnverifiedBackendInputV0` with module-private fields plus an opaque
schema-specific builder or equivalent hierarchical constructors owned by
`src/backend_input.rs`. The construction API exposes only typed V0 record and
scalar inputs needed by the frozen shape; it exposes no public or crate-visible
field bag and no generic `key`, `value`, map, serializer, punctuation, escape,
or writer operation. Unit A's private producer helper builds this model only
after genuine Work Order 19 access. Unit B's decoder builds the same model only
after its ordered occurrence stream has passed closed structural decoding.
Constructing either form remains non-authoritative.

One private `emit_backend_input_v0`-equivalent implementation in
`src/backend_input.rs` owns all payload order, envelope order, punctuation,
number spelling, string escaping, payload hashing, and final-LF emission. It
consumes only the closed model plus a module-private two-case artifact-ID mode:
`ComputeFromPayload` or `PreserveDeclared(&str)`. It has exactly two production
call sites:

1. the Unit A producer wrapper builds the model from authenticated Work Order
   19 facts and calls the emitter with `ComputeFromPayload`; the emitter writes
   canonical payload bytes, computes SHA-256 over those exact emitted bytes,
   and uses that computed ID in the envelope; and
2. the Unit B verifier wrapper accepts the model built from decoded fields and
   calls the same emitter with `PreserveDeclared(decoded_id)` to obtain
   ordinary canonical comparison bytes without recalculation or substitution.

The narrow verifier-facing surface is equivalent to:

```rust
pub(crate) fn reencode_unverified_backend_input_v0(
    model: &UnverifiedBackendInputV0,
    declared_artifact_id: &str,
) -> Option<CanonicalBackendInputArtifact>;
```

The exact opaque builder method split may follow the existing nested V0 shape,
but the top-level ownership and data flow above are frozen. The wrapper may
validate bounded artifact-ID spelling but must not compute, replace, repair,
normalize, or otherwise substitute the declared ID. It returns only ordinary
non-authoritative canonical bytes and range metadata. It cannot return facts
access, a verified range, a permit, `VerifiedBackendInput`, or a callback that
can issue one. `src/backend_input.rs` has no dependency on `src/ir_verify.rs`,
and the verifier has no access to producer facts.

This is one closed model and one writer with two authenticated production call
sites, not friend-style privacy or duplicated canonicalization. Any design that
requires a type cycle, a second emitter, copied spelling logic, generic JSON,
producer authority in the verifier, or a large public field surface is
unsatisfiable and stops Unit B.

The production shape is equivalent to:

```rust
pub(crate) fn with_canonical_minimal_add_artifact<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    item: &Item,
    statement: &ParsedBodyStatement,
    consume: impl for<'facts> FnOnce(
        CanonicalMinimalAddBackendFactsAccess<'facts>,
        CanonicalBackendInputArtifact,
    ) -> R,
) -> Option<R>;
```

That published producer signature, `canonical_minimal_add_artifact`, and the
existing `CanonicalBackendInputArtifact` byte/payload/artifact-ID accessors
remain unchanged. The proven necessary refactor is limited to private producer
assembly/writer internals plus the new opaque model construction surface and
the one crate-private verifier re-encoding wrapper. Any wider producer API
change requires a new BDFL ruling.

`CanonicalBackendInputArtifact` owns only unverified bytes plus non-authority
range metadata needed to locate the payload. It is not a verified capability.
Creating, serializing, deserializing, cloning, or persisting these bytes grants
no backend authority.

The shared `UnverifiedBackendInputV0` and verifier re-encoding result have the
same non-authority status. Building the model, re-emitting it, or possessing
its declared ID never invokes the verifier capability callback and cannot be
converted into `VerifiedBackendInput<'artifact>`. Only `src/ir_verify.rs`,
after canonical equality, digest, structure, semantics, and cross-table checks
all succeed for the caller's original slice, may invoke its one private
capability constructor.

The existing Work Order 19 access and selector names remain available through
narrow `ir_readiness` forwarding/re-export glue where required for compatibility
and compile proofs. There is still one facts producer and one final Program-
lineage validator.

## Verifier and opaque capability

`src/ir_verify.rs` owns the ordered raw-span decoder, canonicality check, digest
check, cross-table semantic verifier, structured report, and sole capability
constructor.

The production shape is equivalent to:

```rust
pub(crate) fn with_verified_backend_input<R>(
    artifact: &[u8],
    consume: impl for<'artifact> FnOnce(VerifiedBackendInput<'artifact>) -> R,
) -> (IrVerifyReport, Option<R>);
```

`VerifiedBackendInput<'artifact>`:

- immutably borrows the exact envelope byte slice passed to the verifier;
- stores only private verified offsets/indices and no caller-controlled permit;
- is constructed in exactly one private success path after every check;
- is non-`Clone`, non-`Copy`, non-`Default`, non-serializable, and has no public
  or crate-visible fields;
- cannot be constructed from source, a JSON value, a decoded model, public
  reports, IDs, strings, or a separately computed digest;
- cannot be rebound to another byte slice;
- cannot outlive the byte owner, become `'static`, or survive process restart;
  and
- exposes only immutable borrowed projections required by IR readiness and a
  future adapter.

The sole constructor is a private associated function whose argument types are
ordinary byte slices and verified offset/range values. The type itself is
crate-visible only so its borrowed projections can cross into IR readiness;
its fields and constructor remain module-private. This precise shape makes the
foreign-construction compile proof and its one-boundary mutation satisfiable.

The exact production constructor surface is:

```rust
impl<'artifact> VerifiedBackendInput<'artifact> {
    fn from_verified_parts(
        artifact: &'artifact [u8],
        payload_range: std::ops::Range<usize>,
        projection_ranges: Vec<std::ops::Range<usize>>,
    ) -> Self;
}
```

There is no second constructor, builder, literal, conversion, default, clone,
or deserialization route. The ordinary range arguments ensure that constructor
privacy itself, rather than an inaccessible parameter type, is the load-bearing
compiler boundary.

The authenticated projections include artifact/payload bytes, artifact ID,
schema/semantic-contract identity, source/module/function identity, ABI subset,
ordered operations/expressions/children, type/definition/effect/resource/
failure-edge references, required passes, and explicit unsupported state.

The verifier's portable notion of Program context is the closed logical context
encoded above. It rejects cross-wired or substituted source/module/item/section/
statement/operation context. It neither serializes nor compares a process-local
Program address. A byte-identical artifact reverified in another process may
receive a new process-local capability; persisted bytes themselves remain
non-authoritative.

Anti-forgery and semantic completeness are separate guarantees:

- anti-forgery comes from the private constructor and byte lifetime; and
- completeness comes from strict canonical parsing, digest validation, closed
  tables, exact cardinality/order, and all cross-table checks.

SHA-256 is a substitution/identity guard, not a signature or proof of who
produced the bytes. A persisted artifact carries no live authority. Another
process must run the same production verifier over its received exact bytes to
obtain a new process-local capability.

## `hum.ir_verify.v0` report

The verifier returns an owned report for human/JSON rendering alongside
callback consumption. The callback completes and its borrowed capability is
dead before the owned tuple is returned. `IrVerifyReport` owns only public
diagnostic values; it contains no borrowed artifact bytes, verified offsets, or
private authority and cannot recreate the capability.

JSON top-level key order is:

1. `schema`
2. `tool`
3. `version`
4. `status`
5. `artifact_schema`
6. `artifact_id`
7. `summary`
8. `rejections`
9. `non_claims_v0`

On accepted minimal add, the exact values of `schema`, `tool`, `version`,
`status`, and `artifact_schema` are respectively `hum.ir_verify.v0`,
`ir-verify`, `0.0.1`, `accepted_canonical_backend_input_v0`, and
`hum.backend_input.v0`. `artifact_id` is the exact envelope ID.

The `summary` object has exactly these keys in order:

1. `payload_bytes`
2. `source_count`
3. `module_count`
4. `function_count`
5. `block_count`
6. `operation_count`
7. `expression_count`
8. `type_count`
9. `definition_count`
10. `effect_count`
11. `resource_count`
12. `failure_edge_count`
13. `required_pass_count`
14. `unsupported_count`

Accepted counts are `1,1,1,1,1,1,1,2,1,1,1,14,0` after the payload-byte
count. The payload-byte count is pinned by the golden artifact rather than
duplicated as an unevaluated planning constant.

`rejections` is empty on success. Each rejection object has exact key order
`code`, `byte_offset`, `logical_path`, `reason`; `byte_offset` may be null only
when no unambiguous byte exists. The schema document must publish the complete
closed code inventory, and every matrix case pins its code. `non_claims_v0` is
exactly the ordered array `not_backend_ready_v0`, `not_executable_v0`,
`not_a_signature_v0`, and `no_durable_authority_v0`.

Success status is `accepted_canonical_backend_input_v0`. Failure status is
`rejected_backend_input_v0`. No field is named `verified`, no boolean claims
durable authority, and `artifact_id` is null only in this report when parsing
fails before a canonical ID can be read. The artifact grammar itself forbids
null.

Every rejection has an ordered stable code, artifact byte offset where known,
logical path, and reason. Multiple independent semantic rejections may be
reported after transport/canonical parsing succeeds. A malformed transport
stops semantic checks because there is no unambiguous model.

Human output presents the same fields in the JSON order: a `Hum IR verify`
heading, schema/tool/version/status/artifact rows, the fourteen summary rows,
ordered rejection rows, then the four non-claims. JSON uses the repository's
ordinary deterministic pretty rendering; it is a report, not canonical input.
For a valid command invocation, accepted and rejected reports go to stdout and
exit zero/one respectively. Invocation and file-I/O errors exit two, go to
stderr, and cannot issue a capability.

## Required rejection classes

The strict production verifier fails closed for:

- invalid UTF-8, BOM, missing final LF, CRLF, extra trailing bytes, or malformed
  JSON;
- canonical-equality failure for otherwise decodable whitespace, escape, or
  alternate scalar spellings, including the two exact stage-5 cases above;
- duplicate/unknown/missing/reordered key or record cardinality drift, detected
  from the retained ordered occurrence stream before map conversion;
- wrong envelope, payload, IR, or semantic-contract version;
- missing, uppercase, short, long, malformed, or mismatched digest;
- exact payload-byte substitution with an unchanged digest;
- source digest/path/file ordinal substitution;
- module ID/name/file-list substitution;
- function/source-item/display/item/linkage substitution;
- ABI convention, parameter/value/type order, result, width, or trap-convention
  substitution;
- block, section, operation, statement, expression, result-value, or provenance
  substitution;
- missing, extra, duplicate, or reordered child;
- foreign, wrong-scope, duplicate, or cross-wired resolver definition;
- type ID/source type/name/kind/signedness/width substitution;
- effect/resource/failure-edge ID or cross-reference substitution;
- any nonempty effect, external authority, move, borrow, alias, transfer,
  allocation, predicate, evidence, or unsupported table;
- allocation declaration or profile substitution;
- wrong overflow type, operation, behavior, duplication, or omission;
- any required pass missing, failed, skipped, zero-selected, unimplemented,
  duplicate, reordered, extra, or foreign;
- facts coherently taken from another operation, source revision, module,
  profile, or target context but cross-wired into this artifact; and
- any missing fact that would require backend inference to reconstruct.

Every semantic corruption must reach the designated production comparison. An
earlier transport failure earns credit only for a transport mutation.

## Production command routes

Two commands are added in order: Unit A adds `backend-input`; Unit B later adds
`ir-verify`.

### `hum backend-input`

```text
hum backend-input examples/core/minimal_add.hum
```

The command accepts exactly one Hum source file. On success stdout is exactly
the canonical artifact envelope bytes, including the one final LF and nothing
else and exit is zero. A valid Hum input that is not the exact supported
minimal-add shape, or that fails an authenticated compiler prerequisite, exits
one with diagnostics on stderr and no artifact stdout. Zero/multiple inputs,
nonexistent or unreadable input, directory input, `--format`, `--timings`, and
unsupported options are invocation or I/O failures: exit two, error on stderr,
and empty stdout.

The command obtains bytes only through the private Work Order 19 production
issuer. It does not render then parse a checker report.

### `hum ir-verify`

```text
hum ir-verify backend-input.json
hum ir-verify --format json backend-input.json
```

These are the only two accepted forms. The command reads exactly one artifact
file as raw bytes and invokes the same production verifier used by IR readiness.
It never routes the file through the Hum source loader or applies text, newline,
path, or source-program normalization. Human output is default; only
`--format json` is accepted, using `hum.ir_verify.v0`.

Exit behavior is closed:

- accepted artifact: exit zero and the selected human/JSON report on stdout;
- artifact rejection: exit one and the selected human/JSON rejection report on
  stdout, including invalid UTF-8, malformed JSON, noncanonical bytes, digest
  mismatch, or semantic invalidity; and
- invocation or I/O failure: exit two, error on stderr, and no report stdout,
  including zero inputs, multiple inputs, nonexistent input, unreadable input,
  directory input, `--timings`, unrelated formats, or unsupported options.

The public report is evidence, not capability authority.

`src/main.rs` must route `ir-verify` before source loading and route
`backend-input` after the ordinary authenticated Hum parse/diagnostic path.
Help, CLI validation, capabilities, and version schema discovery must name both
surfaces.

## IR-readiness consequences by unit

Unit A preserves the blocked state while exposing only completed producer
evidence:

```text
status=blocked_before_ir_verify_with_backend_input_facts_v0
ready_for_ir=0
missing_passes=[ir_verify]
blocking_reasons=[ir_verify_not_implemented]
```

The Unit A exact candidate keeps the existing first nine backend-fact suffix
entries, then appends exactly:

10. `canonical_backend_input_bytes_produced_unverified_v0`
11. `ir_verify_pending_v0`

The golden bytes, artifact ID, and source digest are public evidence only. Unit
A's canonical backend-input selector also freezes this exact readiness order
and blocker set so merely producing or persisting bytes cannot make the
candidate ready; there is no separate readiness selector or extra credit.

Unit B successful verification makes the exact canonical minimal-add operation
honestly IR-ready. It changes only that candidate to:

```text
status=ready_for_ir_with_verified_backend_input_v0
ready_for_ir=1
missing_passes=[]
blocking_reasons=[]
backend_ready=0
backend_blocking_reasons=[backend_adapter_not_implemented]
```

The global summary reports one ready candidate for the standalone fixture and
decrements the blocked count accordingly. All other candidates preserve their
current blocker precedence and readiness values.

The exact candidate's ordered `facts_available` suffix becomes:

1. `canonical_minimal_add_backend_facts_v0`
2. `source_and_operation_identity_bound_v0`
3. `ordered_resolver_bindings_bound_v0`
4. `verified_checked_type_bound_v0`
5. `effect_checked_empty_v0`
6. `ownership_checked_empty_v0`
7. `resource_checked_empty_v0`
8. `normal_profile_checked_v0`
9. `checked_i64_overflow_trap_bound_v0`
10. `canonical_backend_input_bytes_v0`
11. `sha256_payload_identity_verified_v0`
12. `ir_verify_passed_v0`
13. `verified_backend_input_capability_lent_v0`

The Unit B readiness selector compares this complete contiguous ordered suffix,
`missing_passes=[]`, `blocking_reasons=[]`, `backend_ready=0`, and exactly
`backend_blocking_reasons=[backend_adapter_not_implemented]` in both human and
JSON output. Reordering, omitting, duplicating, or renaming a fact, retaining
`ir_verify_pending_v0`, or moving the backend blocker into the IR blocker set
must fail that selector.

`canonical_backend_input_bytes_produced_unverified_v0` is renamed to the
shorter durable completion fact `canonical_backend_input_bytes_v0` only when the
same exact bytes reach successful production verification. `ir_verify_pending_v0`
is removed only for the successful exact candidate.
The pass-status row for `ir_verify` becomes
`implemented_canonical_minimal_add_backend_input_v0`.

`ir_ready=1` means the compiler has one complete, canonical, semantically
verified target-independent backend-input artifact. During computation of that
conclusion, a byte-bound capability was lent only inside the HRTB callback and
was dead before the owned readiness report returned. No capability, borrowed
artifact bytes, verified offsets, or private authority survives in or is
implied by the public report. `ir_ready=1` does not mean backend-ready,
executable, optimized, linked, ABI-stable, cached with authority, or safe for
unsupported language shapes.
The next honest boundary is a separately planned backend adapter that accepts
only `VerifiedBackendInput<'artifact>` and either preserves every fact or
reports loss. Backend adapter and lowering work remain unauthorized.

## Producer, encoder, verifier, capability, consumer map

| Stage and path | Producer | Validator | Consumer |
| --- | --- | --- | --- |
| WO19 chain through `src/profile_check.rs` | exact Program/item/operation semantic authority | existing compiler-sealed stage validators | moved backend-facts issuer |
| `src/backend_input.rs` | private backend facts plus one opaque non-authoritative V0 model | exact cardinality, ordered assignments, provenance, V0 subset | sole canonical emitter |
| `src/sha256.rs` | 32-byte digest of exact payload/source bytes | FIPS vectors and independent .NET comparison | envelope builder and verifier |
| `src/backend_input.rs` emitter | exact model plus compute-or-preserve ID mode | byte grammar, two-call-site audit, golden fixture, declared-ID preservation | producer bytes and verifier comparison bytes |
| `src/ir_verify.rs` decoder | private unverified model from exact bytes | canonicality, digest, closed semantic tables and cross-references | sole capability constructor |
| `src/ir_verify.rs` capability | `VerifiedBackendInput<'artifact>` | private constructor and lifetime | authenticated projections only |
| `src/ir_readiness.rs` | exact candidate readiness row | successful same-byte verifier callback | public readiness projection |
| `src/main.rs` | raw artifact stdout and verifier report routes | exact CLI cardinality/format rules | local humans, CI, future adapters |

No downstream stage reparses Hum source or combines public reports to recreate
authority.

## Complete two-unit implementation envelope

The real dependency and public-consumer audit requires exactly twenty paths
in the Work Order union. `[A]` and `[B]` identify the unit allowed to modify a
path; `[A,B]` is an intentionally shared path modified in both commits:

1. `[A,B]` `src/backend_input.rs` (new in Unit A, shared in Unit B)
   - Unit A owns the moved single backend-facts issuer, deterministic ID
     assignment, and canonical encoder; Unit B refactors only the closed
     unverified model and narrow verifier re-encoding surface while preserving
     the sole writer and every published Unit A byte.
2. `[A]` `src/sha256.rs` (new)
   - implement the one private safe one-shot SHA-256 path and KATs.
3. `[B]` `src/ir_verify.rs` (new)
   - ordered raw-span decode, canonicality, digest/semantic verification,
     report, opaque capability, projections, corruptions, and compile proofs.
4. `[A,B]` `src/ir_readiness.rs`
   - Unit A preserves blocked producer evidence; Unit B consumes the exact
     artifact and callback-scoped capability for the narrow ready state.
5. `[A,B]` `src/main.rs`
   - Unit A declares/routes `backend-input`; Unit B declares/routes raw-byte
     `ir-verify`.
6. `[A,B]` `src/capabilities.rs`
   - register the producer command/schema, then verifier command/schema.
7. `[A,B]` `src/version.rs`
   - report the producer schema, then verifier schema.
8. `[A,B]` `src/ir_contract.rs`
   - advance from produced-unverified to verified target-independent artifact
     while keeping backend lowering planned.
9. `[B]` `src/backend_contract.rs`
   - name `VerifiedBackendInput` as the required future adapter input without
     adding an adapter.
10. `[A]` `docs/HUM_BACKEND_INPUT_SCHEMA.md` (new)
    - publish the exact byte grammar and non-authority rules.
11. `[B]` `docs/HUM_IR_VERIFY_SCHEMA.md` (new)
    - publish report, rejection, capability, and lifetime contracts.
12. `[A,B]` `docs/HUM_IR_READINESS_SCHEMA.md`
    - publish the blocked producer state, then exact ready/backend-blocked state.
13. `[A,B]` `docs/HUM_IR_CONTRACT_SCHEMA.md`
    - publish the produced-unverified boundary, then verified IR boundary.
14. `[B]` `docs/BACKEND_CONTRACT_SCHEMA.md`
    - document the verified-capability adapter prerequisite.
15. `[A,B]` `docs/CAPABILITIES_SCHEMA.md`
    - keep the established public schema/command catalog in exact lockstep
      with each unit's `src/capabilities.rs` state.
16. `[A,B]` `docs/LANGUAGE_REFERENCE.md`
    - keep the established `Current Commands` inventory and bootstrap examples
      in exact lockstep with each unit's real CLI routes and public contracts.
17. `[A,B]` `docs/ARCHITECTURE.md`
    - record each accepted boundary and deferred lowering.
18. `[A,B]` `README.md`
    - Unit A links producer schema/command; Unit B adds verifier schema/command.
19. `[A]` `fixtures/backend_input/minimal_add.backend_input.v0.json` (new)
    - store the exact canonical envelope golden as inspection evidence only.
20. `[A,B]` `tools/check_all.ps1`
    - register unit-specific selectors, compiler proofs, SHA reference checks,
      golden/CLI checks, source audits, compatibility, and deterministic
      evidence.

### Unit A exact subset

Unit A may modify exactly sixteen paths:

```text
src/backend_input.rs
src/sha256.rs
src/ir_readiness.rs
src/main.rs
src/capabilities.rs
src/version.rs
src/ir_contract.rs
docs/HUM_BACKEND_INPUT_SCHEMA.md
docs/HUM_IR_READINESS_SCHEMA.md
docs/HUM_IR_CONTRACT_SCHEMA.md
docs/CAPABILITIES_SCHEMA.md
docs/LANGUAGE_REFERENCE.md
docs/ARCHITECTURE.md
README.md
fixtures/backend_input/minimal_add.backend_input.v0.json
tools/check_all.ps1
```

Unit A's positive evidence owns authenticated fact projection, deterministic
IDs, canonical payload/envelope bytes, source/golden identity, SHA KATs and .NET
comparison, raw `backend-input` stdout, cross-platform byte equality, explicit
unverified status, and the unchanged readiness blocker. Its disposable producer
mutations are exactly:

1. change only SHA-256 round constant `K[0]`; the SHA selector must fail a known
   answer before any verifier exists; and
2. swap only the encoder's envelope emission order for `schema` and
   `artifact_id`; the canonical-byte selector must fail exact golden equality.

Unit A stops if verification/capability code, `ir-verify`, `ready_for_ir=1`, a
seventeenth Unit A path, or a Unit A ceiling breach becomes necessary.

### Unit B exact subset

Unit B starts only from published, terminal-green, status-recorded Unit A and
may modify exactly seventeen paths:

```text
src/backend_input.rs
src/ir_verify.rs
src/ir_readiness.rs
src/main.rs
src/capabilities.rs
src/version.rs
src/ir_contract.rs
src/backend_contract.rs
docs/HUM_IR_VERIFY_SCHEMA.md
docs/HUM_IR_READINESS_SCHEMA.md
docs/HUM_IR_CONTRACT_SCHEMA.md
docs/BACKEND_CONTRACT_SCHEMA.md
docs/CAPABILITIES_SCHEMA.md
docs/LANGUAGE_REFERENCE.md
docs/ARCHITECTURE.md
README.md
tools/check_all.ps1
```

Unit B authenticates published `src/backend_input.rs` blob
`e0ff799e6b2ffb14a23cc3adc5c69218f05e1b12` and may refactor that one Unit A
path only to introduce the opaque shared V0 model and narrow declared-ID-
preserving re-encoding wrapper above. Unit B consumes but does not modify
`src/sha256.rs`, the producer schema, or the golden artifact. Its evidence owns
ordered raw-span decoding, duplicate retention, canonical equality,
digest/semantic verification, all corruption matrices, all existing verifier
mutations plus the shared-interface mutations, the actual-type privacy/lifetime
proof, `ir-verify`, and the readiness transition.

Unit B stops if any published Unit A byte or public interface changes, if the
private refactor exceeds the narrow model/emitter boundary, if the golden
changes, if an eighteenth Unit B path or twenty-first union path is required,
if a capability is needed outside its callback, or if a Unit B ceiling is
breached.

The thirteen intentionally shared paths are exactly:

```text
src/backend_input.rs
src/ir_readiness.rs
src/main.rs
src/capabilities.rs
src/version.rs
src/ir_contract.rs
docs/HUM_IR_READINESS_SCHEMA.md
docs/HUM_IR_CONTRACT_SCHEMA.md
docs/CAPABILITIES_SCHEMA.md
docs/LANGUAGE_REFERENCE.md
docs/ARCHITECTURE.md
README.md
tools/check_all.ps1
```

Unit B may change them only from the exact published Unit A state to the frozen
verifier state described here.

The set arithmetic is exact: `16 + 17 - 13 = 20`. No twenty-first path is
permitted. If any honest implementation requires a path
outside this inventory, stop for a BDFL ruling before editing that path.

### Capability catalog parity by unit

`src/capabilities.rs`, `hum capabilities`, and
`docs/CAPABILITIES_SCHEMA.md` are one ordered public discovery contract. Each
unit changes the code and durable catalog in the same commit and proves exact
human/JSON/document parity with its published command and schema contracts
through the existing capability/schema evidence in `tools/check_all.ps1`. The
evidence extends the existing production inventory; it does not introduce
another parser, detached inventory, selector, or authority route.

Unit A inserts the schema key `backend_input` with exact value
`hum.backend_input.v0` immediately after `ir_readiness` and before
`core_contract` in both human and JSON `schemas` output. It inserts exactly one
command record immediately after `ir_readiness_json` and before
`core_contract_json`:

```text
name=backend_input
command=hum backend-input <file>
schema=hum.backend_input.v0
status=adapter-ready
```

The catalog documents that exact position, spelling, invocation, schema, and
status. Unit A output and documentation contain exactly one `backend_input`
schema registration with value `hum.backend_input.v0` and exactly one
`backend_input` command registration whose invocation contains `backend-input`.
The command record's reference to that same schema is not a second schema
registration. They contain neither an `ir_verify` schema/command registration
nor `hum.ir_verify.v0`. They expose no `VerifiedBackendInput`, verification
claim, or `ir_ready=1`; Unit A remains explicitly unverified.

Unit B preserves those accepted Unit A entries byte-for-byte and inserts the
schema key `ir_verify` with exact value `hum.ir_verify.v0` immediately after
`backend_input` and before `core_contract`. It inserts exactly one command
record immediately after `backend_input` and before `core_contract_json`:

```text
name=ir_verify
command=hum ir-verify [--format json] <backend-input-file>
schema=hum.ir_verify.v0
status=adapter-ready
```

The final catalog and machine-readable output therefore contain exactly one
registration for each of the two commands and two schemas in the frozen order;
each command record refers to its one registered schema. Every preexisting
schema and command entry remains byte-identical and in its prior relative
order. No backend-adapter, ABI, runtime, target, public-crypto, or later
capability may appear.

Per-unit permanent evidence rejects deletion, duplication, substitution,
misspelling, stale documentation, code/document order drift where order is
contractual, or a registration that lacks its real command/schema
implementation. Unit A additionally rejects premature Unit B exposure. Unit B
rejects loss or change of either accepted Unit A entry. Existing capability
schema parity and exact-order checks own this evidence without a sixth WO20
selector or duplicate runtime credit.

### Language-reference parity by unit

`docs/LANGUAGE_REFERENCE.md` is an established public command inventory. Its
`Current Commands` table and its bootstrap examples are two synchronized
surfaces of the same real CLI routes; updating one occurrence while leaving the
other stale is a contract failure. Each unit updates that document in the same
commit as `src/main.rs`, `src/capabilities.rs`,
`docs/CAPABILITIES_SCHEMA.md`, `README.md`, and its unit-specific schema, then
the owning selector and established public-readiness/parity evidence compare
all of those surfaces. No detached inventory, second parser, sixth selector, or
new authority route is introduced.

Unit A adds exactly this one `Current Commands` entry:

```text
hum backend-input <file>
```

It appears immediately after `hum ir-readiness --format json <file-or-dir>...`
and before `hum core-preview <file-or-dir>...`.

It adds exactly this corresponding bootstrap example in the existing bootstrap
block:

```text
cargo run -- backend-input examples/core/minimal_add.hum
```

That bootstrap line appears immediately after
`cargo run -- ir-readiness --format json examples/reference_surface.hum` and
before the first `cargo run -- core-preview` example.

The surrounding prose describes `backend-input` as accepting exactly one Hum
source file and producing canonical but unverified `hum.backend_input.v0` bytes.
It preserves the exact invocation spelling and cardinality and states that the
bytes grant no authority. Unit A's complete language-reference state contains
no `ir-verify`, `hum.ir_verify.v0`, `VerifiedBackendInput`, `ir_ready=1`,
backend-readiness claim, or Unit B authority. It matches the producer route,
producer schema, README, capability catalog, and capabilities output exactly.

Unit B preserves the accepted Unit A command, prose, and bootstrap example and
adds exactly this one `Current Commands` entry:

```text
hum ir-verify [--format json] <backend-input-file>
```

It appears immediately after `hum backend-input <file>` and before
`hum core-preview <file-or-dir>...`.

It adds the two corresponding accepted bootstrap invocations:

```text
cargo run -- ir-verify fixtures/backend_input/minimal_add.backend_input.v0.json
cargo run -- ir-verify --format json fixtures/backend_input/minimal_add.backend_input.v0.json
```

Those two lines appear immediately after the accepted Unit A `backend-input`
bootstrap line and before the first `core-preview` bootstrap example.

Unit B prose states that `ir-verify` verifies exact backend-input bytes without
executing or lowering them, that accepted/rejected/invocation-or-I/O outcomes
exit `0/1/2`, and that `VerifiedBackendInput<'artifact>` exists only during the
production callback. It records the narrow completed state as `ir_ready=1`,
`backend_ready=0`, and
`backend_blocking_reasons=[backend_adapter_not_implemented]`; it does not imply
surviving capability authority or backend execution. The reference must match
README, the capabilities catalog and output, the real route, and
`docs/HUM_IR_VERIFY_SCHEMA.md` exactly.

Unit A permanent evidence rejects a missing, misspelled, duplicated, or
incorrectly positioned `backend-input` entry; incorrect syntax or input
cardinality; a missing or stale bootstrap example; documentation without the
real route; the real route without both documented occurrences; premature
`ir-verify` or `hum.ir_verify.v0`; and any claim that Unit A bytes are verified.
Unit B permanent evidence rejects either command missing; changed or duplicated
Unit A entries; incorrect `ir-verify` syntax; missing, duplicated, or stale
bootstrap examples; drift among language reference, capabilities, README,
schemas, and CLI routing; incorrect `0/1/2` behavior; a backend-ready claim; or
a capability-survival claim. These checks live in the existing owning
selectors and established public-readiness/parity evidence and receive no extra
runtime credit.

### Complete durable-consumer closure

The final independent public-surface audit found no other durable consumer for
these command/schema additions. The closed inventory is:

1. `src/main.rs` for real command declaration, routing, cardinality, and exit
   behavior;
2. `src/capabilities.rs` and `hum capabilities` for machine-readable public
   discovery;
3. `docs/CAPABILITIES_SCHEMA.md` for the durable capabilities catalog;
4. `docs/LANGUAGE_REFERENCE.md` for `Current Commands`, command semantics, and
   bootstrap examples;
5. `README.md` for the public command/schema overview and checked fixture
   mirror;
6. `docs/ARCHITECTURE.md` for the accepted producer/verifier boundary;
7. the unit-specific backend-input, IR-verify, IR-readiness, IR-contract, and
   backend-contract schema documents listed in the envelope;
8. `fixtures/backend_input/minimal_add.backend_input.v0.json` for Unit A's exact
   reviewed bytes; and
9. `tools/check_all.ps1` for load-bearing command/schema/document parity,
   fixture, selector, CLI, and readiness evidence.

This inventory uses existing production and evidence routes. A newly discovered
necessary twenty-first path is an unconditional stop; no public consumer may be
silently omitted, weakened, or replaced with generated or detached inventory.

Specifically excluded are:

- `WORKORDER_19.md`, `WORKORDER_20.md`, governance, decisions, or other Work
  Orders during implementation;
- Cargo manifests or lockfiles, vendored code, licenses, NOTICE, workflows, or
  dependency-audit scripts;
- parser, AST, resolver, type, Core, effect, ownership, resource, or profile
  semantic changes outside the moved existing issuer;
- another fixture, artifact corpus, schema, command, or public document;
- stable global diagnostic-code allocation;
- backend adapters, Cranelift, LLVM, Wasm, C, custom lowering, object emission,
  machine execution, runtime wrappers, linkers, or optimization; and
- cache, signature, provenance, cross-process authority, package, release, or
  trust infrastructure.

The golden artifact is not generated source and must be committed as literal
reviewed bytes. A generated-file workflow is outside scope.

## Exact selector progression

Five new exact selectors are mandatory:

```text
sha256::tests::sha256_known_answer_and_boundary_matrix_is_exact
backend_input::tests::minimal_add_backend_input_bytes_are_canonical_and_deterministic
ir_verify::tests::canonical_backend_input_corruption_matrix_fails_closed
ir_verify::tests::verified_backend_input_is_byte_bound_and_compiler_sealed
ir_readiness::tests::minimal_add_is_ir_ready_only_after_exact_artifact_verification
```

Each independently lists exactly one test, runs exactly one, passes exactly one,
and receives exactly one runtime credit. The pre-Work-Order-20 published
inventory was 99 invocations / 99 unique selectors.

Published Unit A added exactly the first two selectors and is now closed at 101
invocations / 101 unique selectors, with independent named membership for both.
Unit B preserves those 101 credits, adds exactly the final three selectors, and
closes at 104/104 with independent named membership for all five. The shared-
interface evidence adds no sixth selector.

The Unit A backend-input selector also owns the exact blocked readiness proof:
human and JSON output must carry the Unit A status, the exact ordered eleven-
fact suffix, `ready_for_ir=0`, `missing_passes=[ir_verify]`, and
`blocking_reasons=[ir_verify_not_implemented]`. This is part of that selector's
single runtime credit and does not create a third Unit A selector. Unit B's IR-
readiness selector owns the later exact thirteen-fact ready-state transition.

The normal root suite does not substitute for isolated evidence. CLI/golden and
independent SHA reference checks run as separately labeled production-surface
blocks in Fast but do not receive extra exact-selector credits.

## Permanent positive artifact evidence

For the exact fixture, tests prove:

- one source, module, function, `does` section, entry block, return operation,
  and expression;
- two ordered signed-64-bit parameter values;
- one checked `Int` result value;
- two ordered child node/value identities;
- distinct resolver definition and semantic-definition bindings;
- one checked-add operator and one typed checked-overflow trap edge;
- checked-empty effects and external authority;
- checked-empty moves, borrows, aliases, transfers, allocations, predicates,
  evidence, and unsupported state;
- one explicit `allocates: nothing` declaration and accepted `normal` profile;
- the exact fourteen passes once and ordered;
- exact authenticated source/module/item/section/statement/operation
  provenance;
- internal target-independent ABI spellings only;
- exact compact payload and envelope bytes;
- exact source and payload SHA-256 values;
- exact golden file size, payload range, final LF, and artifact ID;
- two encodes of one authenticated input are byte-identical;
- the checked README minimal-add block remains a contiguous fixture mirror;
- human/JSON verifier reports are deterministic and contain no capability;
- no private type, pointer, Program address, permit, or test seam leaks; and
- Windows and Ubuntu produce the same golden bytes and digests.

Changing exactly one authenticated semantic input in a test-owned production
candidate must change payload bytes and digest. This evidence must use a
designated internal corruption seam after genuine Work Order 19 access, not a
second hand-built encoder.

## Canonical transport corruption matrix

Permanent evidence supplies exact byte slices to the production verifier and
rejects each independently:

- empty bytes, invalid UTF-8, BOM, no final LF, CRLF, double LF, and trailing
  byte;
- leading, inter-key, post-colon, post-comma, and pre-closing whitespace;
- envelope key reorder, duplicate, unknown, missing, and extra key;
- payload top-level key reorder, duplicate, unknown, missing, and extra key;
- alternate string escapes, escaped slash, uppercase Unicode hex, escaped
  printable ASCII, surrogate, control byte, leading-zero number, `+0`, `-0`,
  float, and null;
- uppercase/malformed/short/long/missing digest;
- one payload-byte flip with original digest; and
- canonical payload substitution with recomputed digest when the resulting
  semantic model violates the frozen V0 subset.

Each transport case records the exact designated rejection code and proves no
capability callback ran. Syntactically legal whitespace or escape variants are
decoded with raw spans intact and rejected by canonical equality; invalid JSON,
invalid Unicode, forbidden value kinds, duplicates, or closed structural drift
retain earlier designated failures. The two equality-isolation cases are not
credited under any earlier transport code.

## Semantic and cross-table corruption matrix

After transport and digest are independently valid, permanent evidence mutates
and rehashes one semantic fact at a time so the intended semantic comparison
owns rejection:

- every compiler/version/feature/target-context field;
- source ID, source digest, ordinal, path, and module file reference;
- module ID/name and file cardinality/order;
- function ID/source item/display kind/linkage/source span;
- each ABI field and parameter/result order/reference;
- block, section, and operation ID, cardinality, order, kind, statement,
  expression, result, and source span;
- expression ID/kind/operator, every child field/order, result/type/effect/
  resource/failure reference, unsupported state, and provenance;
- type ID/source ID/name/kind/signedness/bits;
- each definition ID/semantic ID/kind/ordinal/value/type/span and cross-wire,
  including the exact two-child aggregate ordered-linkage case;
- effect/resource/failure record ID and every field;
- each empty set independently changed to missing, nonempty, duplicate, or
  reordered where applicable;
- allocation declaration and profile;
- each required-pass name/status/selected/ordinal plus missing, extra,
  duplicate, and reorder; and
- unsupported table omission or nonempty weakened fact.

For each case:

1. exact canonical corrupted bytes reach `ir_verify`;
2. the digest matches those corrupted bytes unless digest mismatch is the
   intended test;
3. the designated semantic rejection owns failure;
4. no earlier syntax/canonicality failure masks it;
5. the capability callback count remains zero; and
6. a public report cannot be converted into a capability.

## Shared canonicalization and non-authority evidence

Unit B permanently proves that the newly shared private refactor preserves the
published producer and creates no second encoder or authority route.

### Unit A byte preservation

Production `hum backend-input examples/core/minimal_add.hum` stdout remains
byte-identical to published Unit A and to the golden. The golden remains exactly
8,715 bytes with raw SHA-256
`9a2affc59962e0d83a33633edce6f318d78a406a9d2e5ad2edc5b8e34cf7c293`.
The payload digest and artifact ID remain
`a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f`.
Both Unit A selectors and all 101 published prior selector credits remain green.
No producer schema, fixture, command/catalog/reference entry, output byte,
source digest, readiness fact, or public API may drift.

### Single-emitter topology

Permanent source/configuration audits require:

- exactly one closed `UnverifiedBackendInputV0` model definition;
- exactly one canonical payload/envelope writer implementation, owned by
  `src/backend_input.rs`;
- exactly one Unit A producer call site using `ComputeFromPayload`;
- exactly one Unit B verifier re-encoding call site using
  `PreserveDeclared(decoded_id)`;
- no canonical punctuation, key-order, number-spelling, string-escaping, or
  final-LF writer in `src/ir_verify.rs`;
- no generic serializer, generic JSON model, map encoder, macro-generated
  duplicate, third production call, or test-only production authority; and
- no dependency from `src/backend_input.rs` to `src/ir_verify.rs`.

Deletion of the shared wrapper, duplication or relocation of the writer,
copied literal emission in `src/ir_verify.rs`, generic serializer substitution,
or a third production call fails the existing Unit B corruption selector or
its owning Fast source-audit block. Hand-built expected canonical strings in
`src/ir_verify.rs` earn no evidence.

### Declared-ID preservation and rejection ownership

A structurally valid envelope whose declared artifact ID is the exact all-zero
lowercase SHA-256 spelling reaches the shared emitter unchanged through
`PreserveDeclared`. Canonical re-emission therefore preserves the wrong ID and
may still equal the original wrong-ID envelope. The verifier's separate digest
comparison, not `src/backend_input.rs`, owns rejection.

A disposable mutation changes only the verifier wrapper's mode from
`PreserveDeclared(decoded_id)` to `ComputeFromPayload`. It must compile and
reach the existing wrong-ID isolation assertion. The mutation masks the
designated digest rejection by repairing the ID, so the corruption selector
must fail. An earlier structural rejection, stale text assertion, missing
symbol, or unrelated failure earns no credit.

### Canonical-equality isolation through the shared emitter

For both exact noncanonical cases--one ASCII space after the payload
`compiler`/`source_revision` comma and one normalized-path slash represented as
`\/`--the production decoder must retain valid structure, produce the same
closed semantic model, observe a matching recomputed digest over the mutated
raw payload, and pass semantic/cross-table checks. The verifier then re-emits
through `src/backend_input.rs`; exact re-encoded-versus-original inequality is
the sole rejection.

A disposable mutation bypasses only this shared canonical re-emission/equality
boundary. Both alternate artifacts must then receive capability access and
fail the corruption selector's zero-callback assertions. A second writer,
hand-built expected string, digest mismatch, syntax failure, semantic change,
or additional bypass earns no credit.

### Shared-model non-authority

Construction, inspection, or canonical emission of
`UnverifiedBackendInputV0` never invokes the capability callback. The model,
declared artifact ID, emitted bytes, artifact value, public report, and range
metadata cannot convert into or construct `VerifiedBackendInput`. Only the
private success path in `src/ir_verify.rs`, after all transport, canonical,
digest, semantic, and cross-table checks, may call
`VerifiedBackendInput::from_verified_parts` over the caller's original byte
slice. Source and compile-proof audits reject any conversion, constructor,
permit, facts access, lineage authority, or callback issuance added to the
shared model or wrapper.

## Load-bearing mutation evidence

Unit A runs the two producer mutations frozen in its path subsection. Unit B's
implementer self-probes, and its fresh reviewer repeats, the four existing
disposable-copy verifier mutations below, with the canonical-byte-equality
mutation now required to bypass the shared `src/backend_input.rs` boundary,
plus the separate declared-ID-preservation mutation above. These five Unit B
mutation classes remain assigned to the existing corruption selector and
source-audit blocks and add no selector credit.

### Digest comparison

The corruption preserves the golden payload bytes exactly and changes only the
envelope's declared `artifact_id` to
`sha256:0000000000000000000000000000000000000000000000000000000000000000`.
The ID is structurally valid lowercase syntax and differs from the golden
digest. Ordered decode, canonical payload, canonical envelope spelling, and all
semantic/cross-table checks pass. Because stage-4 re-encoding preserves the
decoded declared ID, raw-byte equality also passes. The comparison of SHA-256
over the exact raw payload range with the declared ID is the sole rejection.

The disposable mutation bypasses only that computed-versus-declared digest
comparison. The wrong-ID envelope must then receive capability access, causing
`ir_verify::tests::canonical_backend_input_corruption_matrix_fails_closed` to
fail its zero-callback and designated-rejection assertions.

### Ordered child-to-definition linkage

The honest expression children are:

```text
(0, <left-node-id>,  <left-value-id>,  <left-definition-id>)
(1, <right-node-id>, <right-value-id>, <right-definition-id>)
```

The corruption leaves both genuine definition records, their table order,
`ordinal`, IDs, semantic IDs, value IDs, types, spans, uniqueness, and scope
unchanged. It changes only the two child references to:

```text
(0, <left-node-id>,  <left-value-id>,  <right-definition-id>)
(1, <right-node-id>, <right-value-id>, <left-definition-id>)
```

Individual validation proves that each child/node/value/definition exists, both
definitions are distinct and in scope, types match, and all table cardinalities
and ordinals are valid. No individual check associates a child with a
definition. One aggregate production comparison then compares the complete
ordered actual tuple array above with the closed expected array derived from
the decoded definition records at parameter ordinals zero and one and the
corresponding decoded child positions. It does not consult process-local WO19
authority. That aggregate ordered-linkage comparison is the sole rejection.

The disposable mutation bypasses only that aggregate equality. The swapped
artifact must receive capability access and make the corruption selector fail.
Duplicate, missing, foreign-definition, ordinal, table-order, or preselected
error paths earn no credit.

### Canonical byte equality

Two separate corruptions use the exact syntactically decodable variants frozen
under the decoder stages:

1. one ASCII space after the comma between payload members `compiler` and
   `source_revision`; and
2. one `/` in `normalized_path` represented as `\/`.

Each retains the same semantic model. The harness computes SHA-256 over the
mutated raw payload byte range and installs that matching lowercase declared
ID. Decode, duplicate detection, structure, semantics, cross-table checks, and
digest comparison all pass. The closed decoded model then crosses the narrow
verifier wrapper and the sole emitter in `src/backend_input.rs`; canonical
re-encoding removes the selected whitespace/escape variant, making exact
re-encoded-versus-original equality the sole rejection.

The disposable mutation bypasses only that shared canonical re-emission/final
byte-equality boundary. Both alternate envelopes must receive capability
access and make the corruption selector fail. A copied verifier-side encoder,
hand-built expected string, syntax rejection, unchanged digest, semantic
change, or second bypass earns no credit.

### Capability construction

Production defines exactly one module-private
`VerifiedBackendInput::from_verified_parts` constructor. Its parameters are
the original artifact slice plus ordinary owned byte ranges, so a sibling could
call it if and only if the constructor itself became sibling-visible. The
normal actual-type sibling probe names that existing method and fails with the
intended private-associated-function diagnostic, not a missing symbol or
private argument type.

The disposable mutation changes only `fn from_verified_parts` to
`pub(crate) fn from_verified_parts`. The unchanged sibling probe must then
compile and construct a foreign capability over the golden bytes without
running verification. The exact privacy/authority selector, which owns that
compile subprocess and expects construction failure, must fail because the
foreign construction was admitted. Changing only a field, adding a different
test constructor, or retaining another private argument blocker earns no
credit.

Each credited mutation changes one production comparison or visibility
boundary only, compiles, reaches its named assertion, and is removed with all
scratch artifacts. A mutation that fails for syntax, import, missing symbol,
unrelated assertion, stale expected text, or an earlier rejection earns no
credit.

## Actual-type lifetime and privacy proof

One compile-proof configuration uses the real production types and functions.
Normal/failing/normal sequence is exactly `0/101/0`. Named probes include:

```text
verified_backend_input_return_escape_must_not_compile
verified_backend_input_static_escape_must_not_compile
verified_backend_input_collection_escape_must_not_compile
verified_backend_input_foreign_construction_must_not_compile
verified_backend_input_rebind_bytes_must_not_compile
verified_backend_input_from_decoded_report_must_not_compile
verified_backend_input_after_owner_drop_must_not_compile
```

The failing build must contain intended privacy and lifetime diagnostics for all
seven names. Missing symbol/import/cfg, syntax, type inference, move-after-use,
or unrelated diagnostics do not satisfy the proof. The prior `RUSTFLAGS` and
all process-local environment are restored exactly.

Source/configuration audits prove:

- exactly one private capability constructor;
- no `Clone`, `Copy`, `Default`, serializer, builder, conversion, macro, or
  public/crate-visible field path;
- decoder/report types cannot mint or contain the capability;
- the verifier callback is the only issuance route;
- the capability borrows the original caller byte slice, not a re-encoded copy;
- no `cfg(test)` authority route exists in production; and
- deleting test-only corruptions cannot change a normal production build.

## Production and platform configurations

### Normal production

- no test corruption or compile-fail cfg is active;
- the source command emits exact unverified bytes;
- the verifier accepts only exact bytes and lends the capability in callback
  scope;
- IR readiness consumes that same verifier/capability route; and
- no backend call exists.

### Unit test

- corruption state is thread-local and one-shot;
- tests cannot construct facts, artifacts, or capabilities through a second
  production route; and
- parallel execution cannot leak corruption state.

### Compile-fail

- only the named actual-type probes are enabled;
- normal builds before and after remain green; and
- environment restoration is mandatory.

### Platform

Artifact bytes are platform-independent. Required Ubuntu and Windows CI compare
the exact artifact byte length, SHA-256, and golden blob. Only Ubuntu runs the
unrelated platform-independent Exhaustive parser producer; Windows skips only
that duplicate.

## Compatibility and preservation evidence

Each unit compares its exact published parent and candidate behavior in
isolated trees. Unit A may change only the `backend-input` command, producer-
side schemas/contracts, capability-catalog entries, language-reference command
inventory and bootstrap example, the golden artifact, and the frozen unverified
minimal-add readiness state; it does not expose `ir-verify` or final readiness.
Unit B may privately refactor `src/backend_input.rs` only into the shared closed
model/sole-emitter shape while keeping every Unit A byte and public interface
identical. It may otherwise add only the verifier command, verifier-side
schemas/contracts, capability-catalog and language-reference entries, and the
frozen final minimal-add readiness transition. Except for those unit-specific
surfaces, all existing public surfaces are byte-identical.

Required preservation includes:

- source check, graph, resolve, type-env, type-check, Core preview/lower/verify,
  full type, effect, ownership, resource, and profile outputs for minimal add;
- genuine UInt, both legacy additive routes, unsupported target-like,
  integrity-failure, and representative non-target inputs;
- every existing schema not explicitly listed in the envelope;
- blocker precedence and exit behavior for the corpus;
- all Work Order 16, 17, and 19 authority/lifetime evidence;
- exact 99 prior selector credits before adding the five new selectors;
- no private names, addresses, capabilities, or test seams in output; and
- unchanged README fixture parity.

Unit A compares the golden artifact to production `hum backend-input` stdout
and the internal unverified IR-readiness artifact ID. Unit B additionally
compares those same published Unit A bytes to the exact bytes verified by
`hum ir-verify`; redirected `hum backend-input` output must then verify to the
same internal final IR-readiness artifact ID.

## Validation protocol

### Unit A implementer and review

On the exact sixteen-path Unit A candidate, the implementer runs:

- `cargo fmt --all -- --check`, `cargo check --all-targets`, and applicable
  warnings-denied Clippy;
- the two Unit A selectors with list/run/nonzero/credit/membership evidence and
  the closed 101/101 inventory;
- all SHA KATs, checked oversized-input behavior, and independent .NET
  comparison;
- exact source/golden bytes and raw `backend-input` CLI evidence on the
  supported and rejected invocation surfaces;
- the two Unit A disposable producer mutations;
- exact `hum capabilities` human/JSON and
  `docs/CAPABILITIES_SCHEMA.md` parity, including absence of every Unit B
  entry;
- exact `docs/LANGUAGE_REFERENCE.md` `Current Commands` and bootstrap parity
  with the producer route, README, capabilities, and schemas, including absence
  of every Unit B entry;
- source/F4/configuration audits and compatibility proportional to its sixteen
  paths;
- exact blocked readiness status, eleven-fact suffix, and blocker ordering;
- `git diff --check`, text hygiene, public readiness, alpha claims, and release
  readiness for `0.0.1`; and
- one direct Fast on the frozen bytes, with no local Exhaustive.

A fresh Unit A reviewer authenticates its exact base, sixteen paths, blobs,
budgets, marker/stash/archives, reads the full diff, independently reproduces
the source/golden digests and bytes, repeats both selectors and producer
mutations, audits cross-platform source-byte stability, blocked readiness, and
exact capability-catalog and language-reference parity, and issues exactly one
findings-first verdict. At most one direct reviewer Fast is allowed after the
review is otherwise complete.

### Unit A commit, publication, and status

Only after unqualified Unit A acceptance may a separately authorized
implementer stage its exact sixteen paths and create one local commit:

```text
feat(ir): encode canonical backend input bytes
```

Publication is separate. Ubuntu and Windows full CI must test the exact commit,
pass 101/101 selector inventory, the two isolated Unit A selectors, identical
golden/source bytes and digests, .NET reference comparison, exact unverified
CLI/readiness evidence, exact capability-catalog and language-reference parity
without Unit B entries, readiness/public checks, and terminal preflight success.
Ubuntu alone runs the established 14,226-pair Exhaustive producer with seed
`0x48554D5F5345414C`; Windows skips only that duplicate.

A separate status-only edit records the Unit A commit and full CI, proves
immutable-projection identity and fast-lane eligibility, and is separately
published through terminal fast CI. Unit B remains unauthorized until that
status chain is durable and the BDFL sends an explicit Unit B signal.

### Unit B implementer and review

On the exact seventeen-path Unit B candidate based on published Unit A, the
implementer runs:

- `cargo fmt --all -- --check`, `cargo check --all-targets`, and applicable
  warnings-denied Clippy;
- authentication of published `src/backend_input.rs` blob
  `e0ff799e6b2ffb14a23cc3adc5c69218f05e1b12`, complete review of its Unit B
  diff, and exact path/blob/budget telemetry for all seventeen paths;
- the three Unit B selectors plus preservation of Unit A's 101 credits, ending
  at the closed 104/104 inventory and five-name membership;
- ordered/raw-span decoder and duplicate-key source audits;
- proof of exactly one opaque closed V0 model, one canonical emitter, one Unit
  A `ComputeFromPayload` call site, and one Unit B
  `PreserveDeclared(decoded_id)` re-encoding call site, with no copied writer
  logic or generic serializer in `src/ir_verify.rs`;
- published Unit A byte identity: exact production stdout, 8,715-byte golden,
  raw golden SHA-256, payload/artifact ID, both Unit A selectors, and all 101
  prior credits;
- exact accepted/rejected `ir-verify` CLI/report evidence;
- the complete transport and semantic corruption matrices;
- the seven actual-type `0/101/0` lifetime/privacy probes;
- all four existing Unit B disposable mutations plus the declared-ID-
  preservation mutation, including the shared-emitter canonical-equality
  mutation and rejection of a copied verifier-side encoder;
- exact thirteen-fact readiness suffix, empty IR blocker sets, backend-only
  blocker, and callback-death evidence;
- exact final `hum capabilities` human/JSON and
  `docs/CAPABILITIES_SCHEMA.md` parity while preserving both Unit A entries;
- exact final `docs/LANGUAGE_REFERENCE.md` command/prose/bootstrap parity while
  preserving every accepted Unit A occurrence and authority non-claim;
- source/F4/configuration audits and compatibility proportional to its seventeen
  paths;
- `git diff --check`, text hygiene, public readiness, alpha claims, and release
  readiness for `0.0.1`; and
- one direct Fast on the frozen bytes, with no local Exhaustive.

A fresh Unit B reviewer authenticates the published Unit A base, the published
`src/backend_input.rs` starting blob, and all seventeen Unit B
paths/blobs/budgets; reads the complete diff including the newly shared file;
proves the single-emitter/two-call-site topology and Unit A byte identity;
traces model/decoder/verifier/capability/consumer ownership; repeats all three
selectors, the actual-type proof, all four existing mutations, and the
declared-ID-preservation mutation; independently repeats both canonical-
equality cases through the shared emitter; rejects copied canonical spelling in
`src/ir_verify.rs`; checks every corruption for false-early failure; verifies
CLI and readiness lifetime truth; independently verifies capability-catalog
parity plus language-reference parity and preservation; and issues one
findings-first verdict. At most one direct reviewer Fast is allowed after all
other review evidence is complete.

For either unit, a launcher failure before candidate evidence may be repaired
only at the invocation boundary and must be disclosed. A completed Fast failure
stops that unit. Unit B's first Fast did complete red after candidate evidence
began, so its ordinary allowance is consumed permanently. Only the separately
published recovery amendment and all eighteen mandatory recovery gates above
may authorize one new Fast on the two-file corrected candidate; that renewed
run is a distinct event, not a retry or reclassification of the red run.

### Unit B commit, publication, status, and closeout

Only after unqualified Unit B acceptance may a separately authorized
implementer stage its exact seventeen paths and create one local commit:

```text
feat(ir): verify canonical backend input bytes
```

Publication is another BDFL gate. Ubuntu and Windows full CI must test the exact
commit, pass 104/104, all five WO20 selectors, corruption/mutation/compiler
proofs, exact CLI/report/readiness evidence, final capability-catalog parity
and language-reference parity with both Unit A entries preserved, public
readiness, and terminal preflight success. Ubuntu alone runs the established
Exhaustive producer; Windows skips only the duplicate.

Unit B publication does not close Work Order 20. A separately authorized
status-only edit records exact Unit A and B evidence, proves immutable-projection
identity and fast-lane eligibility, and receives separate publication
authorization. Final closeout is separately gated after all evidence is durable.

## Sustainability accounting

The producer and verifier are separable only at one honest boundary: published
canonical bytes that explicitly carry no authority. Unit A cannot claim
verification; Unit B cannot alter the accepted producer bytes. Both remain
narrowed to the exact one-function minimal-add subset.

Each candidate reports per-path and combined telemetry against its own published
parent:

- raw and whitespace-insensitive additions/deletions;
- production Rust insertions;
- permanent test and compile-proof insertions;
- schema/tool/fixture/README/capability-catalog/language-reference insertions;
- moved Work Order 19 facts lines separately from new production logic; and
- exact artifact byte size and golden SHA-256.

Unit A hard ceilings are:

- exactly sixteen Unit A paths;
- at most 950 new or moved production Rust lines;
- at most 650 permanent test/proof Rust lines;
- at most 650 schema/tool/fixture/README/catalog/reference insertions;
- at most 2,250 total insertions;
- at most 660 raw deletions, consisting only of exactly 410 authenticated
  relocation deletions plus at most 250 ordinary/non-relocation deletions;
- exactly one private SHA implementation and one canonical encoder;
- exactly two new selectors and intermediate inventory 101/101; and
- one implementer Fast, at most one complete-review Fast, and no local
  Exhaustive.

Unit B hard ceilings are:

- exactly seventeen Unit B paths based on published Unit A;
- at most 1,800 new or moved production Rust lines;
- at most 1,850 permanent test/compile-proof Rust lines;
- at most 650 schema/tool/README/catalog/reference insertions;
- at most 4,300 total insertions and 550 deletions;
- exactly one ordered raw-span decoder, semantic verifier, report path, and
  capability constructor;
- exactly three new selectors and final inventory 104/104; and
- one implementer Fast, at most one complete-review Fast, and no local
  Exhaustive.

Adding `src/backend_input.rs` transfers no unused Unit A budget. Every Unit B
insertion and deletion in the shared model/emitter refactor, its permanent
tests, and its source-audit evidence counts fully against the Unit B ceilings
above. Published Unit A lines receive no relocation or prior-work discount in
the Unit B diff. If the one-model/one-emitter/two-call-site design cannot fit
honestly, Unit B stops with exact measured evidence rather than weakening
canonicality, declared-ID preservation, or capability authority.

The non-transferable union ceilings are therefore twenty paths, 2,750
production Rust insertions, 2,500 test/proof insertions, 1,300
schema/tool/fixture/README/catalog/reference insertions, 6,550 total
insertions, and 1,210 raw deletions. Exactly 410 of those raw deletions are the
authenticated Unit A relocation allowance, so the relocation-adjusted union
deletion ceiling remains 800. Unit B retains its independent 550-raw-deletion
ceiling. Unused Unit A insertion, ordinary-deletion, or relocation budget
cannot be transferred to Unit B, and unused Unit B budget cannot transfer to
Unit A.
The newly recognized durable consumers fit inside each existing documentation/
tool category ceiling; neither per-unit nor union line budget increases.
Across both units there remains exactly one SHA implementation, one closed V0
model, one canonical encoder, one strict decoder, one semantic verifier, and
one capability constructor, with no dependency, unsafe code, build script,
generated source, or duplicated public projection. The exact set arithmetic is
`16 + 17 - 13 = 20`; the established non-transferable union ceilings do not
change.

These are review ceilings, not targets. Normal rustfmt output is mandatory; no
format suppression or line packing may evade them. If either honest unit
exceeds its own ceiling, stop with measured evidence. Do not weaken producer
determinism, verification, capability safety, or the published boundary to fit.

## Explicit exclusions and bans

Work Order 20 does not authorize:

- general Hum IR or artifacts beyond exact canonical minimal add;
- public stable ABI, platform ABI, object layout, target triple, calling into a
  backend, or code execution;
- Cranelift, LLVM, Wasm, C, custom backend, interpreter expansion, optimizer,
  linker, runtime wrapper, object file, or executable;
- artifact cache as authority, cross-process capability transfer, signature,
  signing key, certificate, transparency log, provenance, or trust system;
- public SHA or crypto API;
- package/dependency introduction, Cargo changes, vendoring, unsafe, FFI,
  build script, proc macro, or generated code;
- generalized JSON library, generalized serialization framework, or arbitrary
  schema support;
- recovery-stash cleanup, archive mutation, native-stderr harness repair,
  open-skeleton integration, termination work, release/tag work, or unrelated
  cleanup; or
- another Work Order, governance change, or later backend planning.

No public report, golden file, artifact ID, cached bytes, or serialized success
row may construct or stand in for `VerifiedBackendInput<'artifact>`.

## Mandatory stop conditions

Stop without workaround if:

- a required semantic fact is absent from the accepted WO19 authority;
- production needs public JSON, source-text reconstruction, an independent
  report join, pointer serialization, or caller-supplied identity;
- any twenty-first path is necessary;
- Cargo, license, vendoring, workflow, or dependency files become necessary;
- a second SHA, encoder, decoder, verifier, or capability constructor appears;
- a second canonical writer, verifier-side copied punctuation/key-order/
  escaping logic, a generic serializer, or a generic JSON model appears;
- the verifier wrapper recomputes, replaces, repairs, or normalizes the decoded
  declared artifact ID before canonical equality;
- the shared unverified model, its builder, emitted bytes, artifact value, or
  verifier wrapper grants facts, lineage, permit, callback, or capability
  authority;
- semantically equivalent noncanonical bytes receive a capability;
- a digest or cross-table mutation can receive capability without failing its
  exact selector;
- the capability can escape, be rebound, serialized, cloned, constructed by a
  sibling, or recreated from a report;
- artifact output differs across Ubuntu and Windows;
- readiness says one before the exact verifier callback or says backend-ready;
- a backend call, lowering, target artifact, or execution enters;
- any existing nonminimal public behavior changes outside the frozen envelope;
- Unit A exceeds its sixteen paths or budget, or claims verification/readiness;
- Unit B changes any published Unit A output byte, golden identity, source
  digest, artifact ID, producer schema, CLI/catalog/reference surface, selector
  behavior, or public interface;
- Unit B exceeds its seventeen paths or budget, or requires an eighteenth Unit
  B path;
- a selector selects zero or multiple tests;
- Fast fails after candidate evidence begins; Unit B's historical run did fail
  and stopped the unit, and any separately authorized renewed recovery Fast
  that completes red also stops without repair or retry; or
- either unit cannot be independently reviewed in one sitting.

## Work Order 20 lifecycle

The gates are separate and no gate implies the next:

1. the initial independent planning review, bounded author correction, and
   terminal corrected-document review;
2. the first BDFL-authorized terminal envelope amendment adding the capabilities
   catalog and its independent terminal-amendment review;
3. this final BDFL-pinned re-envelope adding the language reference and one
   final fresh independent review;
4. explicit BDFL acceptance and local documentation commit;
5. separate BDFL planning publication and terminal full CI;
6. isolated planning publication-status record and terminal fast CI;
7. explicit BDFL Unit A implementation signal;
8. the first bounded sixteen-path Unit A implementation, its valid completed-
   red Fast stop, and lossless candidate parking;
9. this relocation-aware amendment, fresh independent review, separate local
   commit, full-CI publication, status record, and fast-CI status publication;
10. separate explicit BDFL resumption, exact candidate restoration, bounded
    tool-audit correction, one renewed Fast, and fresh complete Unit A review;
11. at most one BDFL-authorized bounded Unit A implementation correction after
    that review;
12. separately authorized Unit A local commit;
13. separately authorized Unit A publication and terminal 101/101 full CI;
14. isolated Unit A status record and terminal fast CI;
15. the original explicit BDFL Unit B implementation signal and the correct
    pre-implementation stop at the private encoder-interface defect, with no
    candidate or evidence run consumed;
16. this document-only encoder-interface amendment;
17. one fresh independent amendment review, where only unqualified `ACCEPT`
    advances;
18. a separately authorized local amendment commit containing only
    `WORKORDER_20.md`;
19. separate amendment publication and terminal-green full CI;
20. an isolated amendment publication-status commit;
21. separate status publication and terminal-green fast CI;
22. the explicit BDFL Unit B resumption signal;
23. the bounded seventeen-path Unit B implementation on the published amendment
    status baseline, its completed-red Fast after candidate evidence began, and
    lossless parking at stash commit
    `303ee9af93696409bea66d3f8a379cb1a8cf8e1a`;
24. this document-only completed-red Fast-boundary recovery amendment, its two
    independent review findings, and their bounded document corrections;
25. one fresh independent final corrected recovery-amendment review, where
    only unqualified `ACCEPT` advances;
26. a separately authorized local recovery-amendment commit containing only
    `WORKORDER_20.md`;
27. separate recovery-amendment publication and terminal-green full CI;
28. an isolated recovery-amendment publication-status commit;
29. separate status publication and terminal-green fast CI;
30. a new explicit BDFL Unit B recovery signal;
31. exact stopped-stash restoration without stash consumption or reordering,
    authentication of all seventeen blobs, and exactly the frozen
    `docs/CAPABILITIES_SCHEMA.md` and `tools/check_all.ps1` corrections;
32. focused honest catalog evidence, all six document variants, the
    audit-weakening mutation, proportional checks, and exactly one renewed
    direct Fast;
33. on terminal-green Fast only, one fresh independent complete Unit B review;
34. on unqualified Unit B `ACCEPT` only, separately authorized Unit B local
    commit;
35. separately authorized Unit B publication and terminal 104/104 full CI; and
36. separately authorized final status/closeout record and publication.

The planning correction cycle remains consumed. The capabilities-catalog and
language-reference re-envelopes, plus this BDFL-directed Fast-boundary
accounting amendment, are recorded exceptions and create no automatic
successor correction. Any finding in the fresh amendment review returns
directly to the BDFL without another author edit. Each later implementation
correction allowance is local to its own unit and cannot add a path,
dependency, mechanism, public contract, evidence meaning, or backend behavior.
A repeated authority/evidence failure or envelope defect returns directly to
the BDFL. The first Unit B Fast allowance was consumed by the completed-red
candidate event at step 23. The single renewed Fast at step 32 is new authority
created only by the complete recovery chain; it never alters the historical
red result.

## Planning-package validation

The architect-author runs only document-level evidence:

- `git diff --check`;
- fail-closed no-index whitespace checking for `WORKORDER_20.md`;
- complete 123-case status-classifier suite twice with byte-identical output;
- text hygiene for 532 files;
- public readiness for 532 files;
- alpha claims; and
- release readiness for `0.0.1`.

The two credited classifier processes use exactly:

```text
powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/test_workorder_status_boundary.ps1
```

Each child has stdout and stderr redirected through
`System.Diagnostics.ProcessStartInfo`; the author copies
`StandardOutput.BaseStream` and `StandardError.BaseStream` directly into
memory, without text decoding, line splitting, `Write-Output`, file rewriting,
or newline normalization. Exit codes, raw byte lengths, CR/LF counts, and
SHA-256 of each raw stream are reported before comparison. A separately
calculated LF-to-CRLF expansion may explain a historical wrapper discrepancy
but receives no credited identity and does not alter the classifier.

No Cargo command, Rust selector, Fast, full preflight, Exhaustive, workflow,
performance check, implementation probe, archive restoration, stash operation,
commit, or push is part of planning validation.

## Current authorization gate

The independently accepted Unit B Fast-boundary recovery amendment is
published as `74eb0396a19ea1a058bd3fed05939c1cda7ba5a5` by a normal non-force
fast-forward of `main` only. Workflow `ci`, run `31667895670`, attempt 1,
tested that exact SHA and concluded `success`. Ubuntu job `94346311867` and
Windows job `94346311903` both completed the full lane successfully with
`mode=full;reason=no_status_transition`, exact selector inventory `101/101`,
532-file text-hygiene and public-readiness evidence, alpha claims, release
readiness `0.0.1`, and exactly one terminal preflight-success marker. Ubuntu
completed all 14,226 Exhaustive cases with seed `0x48554D5F5345414C`;
Windows correctly skipped the duplicate producer.

Unit B remains stopped and parked at stash commit
`303ee9af93696409bea66d3f8a379cb1a8cf8e1a`. Candidate restoration and both
frozen corrections remain unauthorized. This local status record does not
accept Unit B, authorize an implementation commit, or imply any later work.

The next gates remain exact and separate:

1. separately authorized publication of this status commit;
2. terminal-green fast-lane CI for that exact publication;
3. a fresh explicit BDFL Unit B recovery/resumption signal; and
4. only after that signal, exact restoration of the parked stash and exactly
   the two frozen corrections to `docs/CAPABILITIES_SCHEMA.md` and
   `tools/check_all.ps1` under the established recovery lifecycle.

Renewed validation, Fast, independent Unit B review, implementation commit,
publication, closeout, stash cleanup, archive mutation, Work Order
organization, semantic-coordinate research, and every later backend activity
remain unauthorized.
<!-- workorder-current-authorization-gate:end -->

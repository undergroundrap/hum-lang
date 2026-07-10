use crate::diagnostic::{DiagnosticCode, Severity};

pub const DIAGNOSTIC_EXPLAIN_SCHEMA: &str = "hum.diagnostic_explain.v0";
pub const DIAGNOSTIC_CATALOG_SCHEMA: &str = "hum.diagnostic_catalog.v0";

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticInfo {
    pub code: DiagnosticCode,
    pub default_severity: Severity,
    pub explanation: &'static str,
    pub repair: &'static str,
}

pub const DIAGNOSTICS: &[DiagnosticInfo] = &[
    DiagnosticInfo {
        code: DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE,
        default_severity: Severity::Warning,
        explanation: "A top-level line does not start a recognized Hum item. Milestone 0 only recognizes module declarations and app, type, store, task, or test items.",
        repair: "Move the line into a recognized item body, turn it into a section line, or remove it until the surface grammar supports it.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::NESTED_ITEM_EXTENDS_PAST_BLOCK,
        default_severity: Severity::Error,
        explanation: "A nested item crosses the closing brace of the app or item that contains it, so the parser cannot trust the block structure.",
        repair: "Check the surrounding braces and keep the nested item fully inside its parent block.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE,
        default_severity: Severity::Error,
        explanation: "An item header is missing the opening brace that starts its body.",
        repair: "Add `{` at the end of the item header or rewrite the line as section text inside an existing item.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ITEM_BLOCK_MISSING_CLOSE_BRACE,
        default_severity: Severity::Error,
        explanation: "An item body starts but does not close before the file ends or its parent block ends.",
        repair: "Add the missing `}` at the intended end of the item body.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_ITEM_KIND,
        default_severity: Severity::Warning,
        explanation: "The parser found an item-like header whose first word is not a current Hum item kind.",
        repair: "Use `app`, `type`, `store`, `task`, or `test`, or keep the idea in a checked section until the item kind is designed.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNEXPECTED_SIGNATURE_TEXT,
        default_severity: Severity::Warning,
        explanation: "A task or test signature has trailing text after the part Milestone 0 understands.",
        repair: "Move the extra text into a checked section or simplify the signature to the current parser contract.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN,
        default_severity: Severity::Error,
        explanation: "A task or test parameter list starts with `(` but does not close with `)`.",
        repair: "Close the parameter list or remove the opening parenthesis if the item takes no parameters.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::PARAMETER_MISSING_TYPE,
        default_severity: Severity::Error,
        explanation: "A parameter does not declare an explicit type, so later checks cannot know what the callable expects.",
        repair: "Write the parameter as `name: Type`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_IDENTIFIER,
        default_severity: Severity::Error,
        explanation: "A Hum identifier uses one deterministic token. Value names use snake_case and type names use PascalCase, so spaced names cannot be parsed as symbols.",
        repair: "Use a single token such as `remember_work_item`; put human phrasing in `why:` or another prose section.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_MISSING_WHY,
        default_severity: Severity::Warning,
        explanation: "An app lacks a `why:` section, leaving the application purpose invisible to readers and tools.",
        repair: "Add a `why:` section that states the app's purpose in plain language.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TYPE_MISSING_SHAPE,
        default_severity: Severity::Warning,
        explanation: "A type has no visible fields or shape facts, so it does not yet explain what data it represents.",
        repair: "Add fields or an intent section that describes the type's shape.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::STORE_MISSING_TYPE,
        default_severity: Severity::Warning,
        explanation: "A store does not declare the type of data it contains.",
        repair: "Write the store header with a type, such as `store sessions: list Session`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::STORE_MISSING_PURPOSE,
        default_severity: Severity::Warning,
        explanation: "A store lacks visible purpose, which makes persistence and shared state harder to review.",
        repair: "Add `why:` or another accepted purpose section explaining what the store remembers and why.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MISSING_REQUIRED_SECTION,
        default_severity: Severity::Error,
        explanation: "An item is missing a section Milestone 0 needs, such as `does:`, or nontrivial behavior is missing visible purpose in `why:`.",
        repair: "Add `does:` when the body is missing. Add `why:` when effects, failures, or body size make purpose non-obvious.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::DUPLICATE_SECTION,
        default_severity: Severity::Warning,
        explanation: "The same section appears more than once in one item, which can split important intent across multiple places.",
        repair: "Merge the repeated sections into one section in canonical order.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TASK_MISSING_NEEDS,
        default_severity: Severity::Warning,
        explanation: "A task has a risky boundary but no `needs:` section, so real caller responsibilities may be hidden.",
        repair: "Add `needs:` only when callers must satisfy a real precondition; avoid filler for pure local code.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::SECTION_OUT_OF_ORDER,
        default_severity: Severity::Warning,
        explanation: "A known section appears after a later section in Hum's canonical review order.",
        repair: "Move the section into the documented order so readers see purpose, capabilities, contracts, risks, cost, and implementation consistently.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TASK_MISSING_ENSURES,
        default_severity: Severity::Warning,
        explanation: "A returning task crosses a nontrivial boundary without an `ensures:` section, so its success promise may be hidden.",
        repair: "Add `ensures:` when the result promise is not obvious from a small pure body.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::HOLLOW_CONTRACT_LINE,
        default_severity: Severity::Warning,
        explanation: "A contract-like section contains a line that is too generic, tautological, or placeholder-shaped to reject a meaningful wrong implementation.",
        repair: "Replace it with a specific claim, edge case, boundary, allocation limit, or success condition that could catch a real mistake.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNDECLARED_SAVE_TARGET,
        default_severity: Severity::Error,
        explanation: "A `does:` body saves into a resource that is not listed under `changes:`, so mutation would be hidden from readers and tools.",
        repair: "Add the resource under `changes:` if the mutation is intended, or remove the save.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNDECLARED_SET_TARGET,
        default_severity: Severity::Error,
        explanation: "A `does:` body sets a name that is neither a local `change` binding nor a declared changed resource.",
        repair: "Declare the local name with `change`, list the resource under `changes:`, or avoid the mutation.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TASK_MISSING_COST,
        default_severity: Severity::Warning,
        explanation: "A task has loops, effects, or a larger body but no `cost:` section, so resource expectations are not visible for review.",
        repair: "Add `cost:` when resource behavior is worth reviewing; avoid filler for tiny pure tasks.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::COST_MISSING_CHECK,
        default_severity: Severity::Warning,
        explanation: "A `cost:` block lacks a `check:` level, so tools do not know whether the claim is informational or enforced.",
        repair: "Add `check: warn`, `check: compile`, or another accepted check level as the cost model matures.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::COMPILE_COST_MISSING_TIME,
        default_severity: Severity::Error,
        explanation: "A task asks for compile-time cost checking but does not state a `time:` claim.",
        repair: "Add a `time:` claim such as `time: O(1)` or lower the check level until the cost can be stated honestly.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CONSTANT_COST_HAS_FOR_EACH,
        default_severity: Severity::Error,
        explanation: "A task claims constant time while visibly using `for each`, which usually depends on input size.",
        repair: "Change the time claim, remove the iteration, or record a narrower bound with a clear reason.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::COMPILE_COST_UNBOUNDED_WHILE,
        default_severity: Severity::Error,
        explanation: "A compile-checked cost claim contains a `while` loop without an obvious bound.",
        repair: "Use a visibly bounded loop, add a clearer bound, or lower the check level until the checker can prove the claim.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::SECURITY_MISSING_PROTECTS,
        default_severity: Severity::Warning,
        explanation: "A task touches security-sensitive names but does not state what it protects.",
        repair: "Add a `protects:` section that names the security property, or remove the sensitive dependency.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TRUSTS_MISSING_PROTECTS,
        default_severity: Severity::Warning,
        explanation: "A task declares a trust boundary without a matching safety or security promise.",
        repair: "Add `protects:` lines that say what must remain true across the trusted boundary.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TEST_MISSING_COVERS,
        default_severity: Severity::Warning,
        explanation: "A test does not say which promise it covers, so it cannot serve as first-class evidence.",
        repair: "Add a `covers:` section that points at the task obligation or promise the test checks.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::REGRESSION_MISSING_NOTE,
        default_severity: Severity::Warning,
        explanation: "A regression test does not record the bug shape it prevents from returning.",
        repair: "Add a `regression:` section describing the old failure mode.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNRESOLVED_NAME,
        default_severity: Severity::Error,
        explanation: "A checked resolver found a name that is not visible from the current lexical scope or declared dependency boundary.",
        repair: "Declare the name before use, add a matching item, or list the external dependency under `uses:` when it is intentional.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::DUPLICATE_NAME_IN_SCOPE,
        default_severity: Severity::Error,
        explanation: "Two definitions in one scope normalize to the same name, so references would be ambiguous.",
        repair: "Rename one binding or move it into a narrower block so the scope has one definition for the name.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::SET_TARGET_IMMUTABLE,
        default_severity: Severity::Error,
        explanation: "A `set` target resolves to a visible definition, but that definition is not a mutable place or declared change permission.",
        repair: "Declare the local with `change`, target a declared `changes:` permission, or keep the value immutable.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::READ_BEFORE_DECLARE,
        default_severity: Severity::Error,
        explanation: "A name is read before its later local declaration, which makes the data dependency order unclear.",
        repair: "Move the declaration above the read or pass the value in through an explicit parameter.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_TYPE_NAME,
        default_severity: Severity::Error,
        explanation: "A declaration annotation names a type that is neither declared in the source nor reserved by the current Hum type environment.",
        repair: "Declare the type, use a reserved type root, or wait for imports/packages before relying on external type names.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RETURN_TYPE_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A task return expression has a trivial source-visible type that does not match the task result type or Result/Option/Maybe success type.",
        repair: "Return the expected value type, change the task result annotation, or keep complex expressions unchecked until full expression typing exists.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_MISSING,
        default_severity: Severity::Error,
        explanation: "A top-level executable app has no `starts with:` declaration, so `hum run` has no structural task root.",
        repair: "Add exactly one `starts with:` section containing one bare name of a task directly nested in the app.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_EMPTY,
        default_severity: Severity::Error,
        explanation: "An executable app has a `starts with:` section but no meaningful start-task name.",
        repair: "Put exactly one bare snake_case direct-child task name under `starts with:`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_DUPLICATE,
        default_severity: Severity::Error,
        explanation: "An executable app has multiple `starts with:` sections or multiple meaningful start declarations, making its root ambiguous.",
        repair: "Keep one `starts with:` section with one meaningful bare task name.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_INVALID_NAME,
        default_severity: Severity::Error,
        explanation: "An app start declaration is not one bare snake_case value identifier.",
        repair: "Use a direct-child task name such as `run_tool`, without call syntax, paths, assignments, or state initialization.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_NOT_CHILD,
        default_severity: Severity::Error,
        explanation: "An app start name does not resolve to a task directly nested in that app. App mode never falls back to global task lookup.",
        repair: "Nest the named task directly in the app or name an existing direct child.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MULTIPLE_EXECUTABLE_APPS,
        default_severity: Severity::Error,
        explanation: "Run input contains more than one top-level app, so there is no unique executable program root.",
        repair: "Run exactly one top-level app input, or use `--entry <task>` for a direct legacy task probe.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_START_INVALID_RESULT,
        default_severity: Severity::Error,
        explanation: "An app start task does not return `Unit` or `Result Unit, E`, so app completion would expose an unsupported program result.",
        repair: "Return `Unit` (including an omitted result) or `Result Unit, ErrorType` from the start task.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_SOURCE_CAPABILITY,
        default_severity: Severity::Error,
        explanation: "An executable app or one of its tasks uses a dotted external-capability spelling outside Session Y's exact `stdout.write`, `clock.replay`, and `files.read` vocabulary.",
        repair: "Use one exact pinned capability ID, or keep an ordinary non-capability dependency under `uses:` without an external capability-family spelling.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MISSING_CALLER_CAPABILITY,
        default_severity: Severity::Error,
        explanation: "A caller reaches a callee capability closure that is not included in the caller's own exact source-authority budget.",
        repair: "Add the exact capability under the caller's `uses:` section or remove/restructure the call path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::APP_CAPABILITY_MISMATCH,
        default_severity: Severity::Error,
        explanation: "The app maximum source-authority declaration does not cover a pinned capability in the start task's closed direct-call route.",
        repair: "Add the exact capability under the app's `uses:` section or remove it from the reachable start-task closure; source declaration is not operator consent.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ENTRY_CAPABILITY_BYPASS,
        default_severity: Severity::Error,
        explanation: "An explicit `--entry` task reaches pinned external source authority, but direct entry is a pure probe outside the structural app authority root.",
        repair: "Run the structural app without `--entry`, or select a task whose complete direct-call closure is pure.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::OUTPUT_CAPABILITY_UNDECLARED,
        default_severity: Severity::Error,
        explanation: "A `stdout_write` call is executable only when both its task and containing app declare exact `stdout.write` source authority.",
        repair: "Add exact `stdout.write` to the blamed task and app `uses:` sections, preserving caller closure, or remove the output call.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INVALID_STDOUT_WRITE_CALL,
        default_severity: Severity::Error,
        explanation: "The bounded output built-in has exactly one signature: `stdout_write(text: Text) -> Result Unit, OutputError`.",
        repair: "Pass exactly one checked `Text` argument and handle the typed `OutputError` with the authorized explicit propagation or wrapping form.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RESERVED_BUILTIN_NAME,
        default_severity: Severity::Error,
        explanation: "A user task redeclares the exact `stdout_write` name reserved for Hum's bounded output built-in, which would split callable identity across stages.",
        repair: "Rename the user task and keep `stdout_write` reserved for `stdout_write(text: Text) -> Result Unit, OutputError`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::OUTPUT_RECURSION_UNSUPPORTED,
        default_severity: Severity::Error,
        explanation: "A recursive call cycle is reachable from the structural app start and can reach bounded output, but Session Z audit routes require finite exact call-occurrence identities.",
        repair: "Rewrite the output-bearing recursion as an explicit bounded loop or a non-recursive task chain so every output route has a finite auditable call path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNCHECKED_PROSE_CONTRACT,
        default_severity: Severity::Warning,
        explanation: "`hum run` saw a `needs:` or `ensures:` line that is honest prose rather than a predicate v1 expression, so it remains visible but unchecked. Predicate v1 is one comparison over parameters, `result`, arithmetic, `list_len(...)`, and `old(...)` of entry-readable parameter places in `ensures:` only.",
        repair: "Use one canonical comparison such as `b != 0`, `result == a + b`, `result.title == old(item.title)`, or `list_len(result) == 3` when the contract is meant to execute now; keep prose when it is intentionally unchecked.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::NEEDS_CONTRACT_VIOLATION,
        default_severity: Severity::Error,
        explanation: "A runtime `needs:` predicate evaluated to false at task entry, so the caller or calling context broke the precondition.",
        repair: "Change the call arguments or add a caller-side guard before invoking the task.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ENSURES_CONTRACT_VIOLATION,
        default_severity: Severity::Error,
        explanation: "A runtime `ensures:` predicate evaluated to false after a successful return, so the task implementation broke its own success promise.",
        repair: "Fix the task body or adjust the contract to match the intended result.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::USE_AFTER_MOVE,
        default_severity: Severity::Error,
        explanation: "A value was used after an earlier `consume` argument or return moved its ownership authority away from the current place.",
        repair: "Use the value before the move site named in the help text, or create and pass a fresh owned value.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::BORROW_PARAMETER_MUTATION,
        default_severity: Severity::Error,
        explanation: "A parameter with default `borrow` permission was targeted by `set` or used to acquire a writable field alias, which would hide mutation behind a read-only signature.",
        repair: "Mark the parameter `change`, copy into a `change` local before acquiring the alias or writing, or remove the mutation.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED,
        default_severity: Severity::Error,
        explanation: "A recognized linear resource reached a return, failure, or fallthrough path without exactly one visible `consume` action.",
        repair: "Commit, rollback, close, or transfer the resource exactly once on the path named in the help text.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE,
        default_severity: Severity::Error,
        explanation: "A recognized linear resource was consumed after an earlier commit, rollback, close, transfer, or consume already ended it on that path.",
        repair: "Remove the second consume, split the control flow, or create a fresh resource before consuming again.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER,
        default_severity: Severity::Error,
        explanation: "A returned-view `from` relationship names a source that is not a task parameter, or the returned expression does not visibly come from that source.",
        repair: "Name a parameter after `from` and return that bare parameter in the V0 subset; locals, internal references, and complex expressions remain explicit future repairs.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::ITERATION_MUTATION_CONFLICT,
        default_severity: Severity::Error,
        explanation: "A list was structurally mutated while a `for each` loop was actively iterating that same collection.",
        repair: "Collect mutations after the loop, or iterate over a separate snapshot/list before calling `list_append` on the original collection.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::STALE_FIELD_VIEW,
        default_severity: Severity::Error,
        explanation: "A local field or element view was used after the source it viewed was changed by a recognized invalidating operation.",
        repair: "Re-borrow the view after the write or append, or bind a value copy before the invalidating operation if stale observation was intended.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::WRITABLE_ALIAS_OVERLAP,
        default_severity: Severity::Error,
        explanation: "A direct field is accessed through another path while a writable alias to overlapping storage is still live.",
        repair: "Use a definitely distinct direct field, or move the access after the writable alias's last syntactic use.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS,
        default_severity: Severity::Error,
        explanation: "A writable alias uses a shape or lifetime outside Hum's narrow straight-line direct-field slice.",
        repair: "Keep the alias local and straight-line over one direct field; do not pass, return, store, nest, rebind, or use it across control flow.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
        default_severity: Severity::Error,
        explanation: "A known task with a nominal error result was called without explicit propagation or wrapping.",
        repair: "Write `try call()` for a matching caller error root, or `try call() or fail CallerError.context` to preserve and wrap the cause.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION,
        default_severity: Severity::Error,
        explanation: "An unwrapped `try` would propagate a callee error root different from the caller's declared error root.",
        repair: "Wrap the cause under the caller's root or make the nominal roots equal deliberately.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::FAILURE_WRAPPER_ROOT_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A causal wrapper uses a nominal root different from the caller's declared error root.",
        repair: "Use a wrapper variant under the caller's declared error root.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::TRY_ON_INFALLIBLE_CALL,
        default_severity: Severity::Error,
        explanation: "`try` was applied to a known task whose result has no typed error root.",
        repair: "Remove `try` and call the infallible task directly.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::DIRECT_FAILURE_ROOT_MISMATCH,
        default_severity: Severity::Error,
        explanation: "A direct `fail Root.variant` uses a root different from the enclosing task result.",
        repair: "Fail with a variant under the caller's declared error root or change the result deliberately.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNSUPPORTED_TRY_EXPRESSION,
        default_severity: Severity::Error,
        explanation: "A `try` expression is outside the exact direct named-call Session W slice.",
        repair: "Use `let value = try named_call(...)` with ordinary value arguments, optionally followed by `or fail CallerError.context`.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::MISSING_FAILURE_DECLARATION,
        default_severity: Severity::Error,
        explanation: "A task can directly fail, propagate, or wrap a typed failure but has no meaningful `fails when:` declaration.",
        repair: "Add a concrete `fails when:` condition for the visible failure path.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_TARGET_FACT_RECORD,
        default_severity: Severity::Error,
        explanation: "A `targets:` section names a target fact record that Hum does not publish in `hum target-facts`.",
        repair: "Use one of the fixture record IDs from `hum target-facts --format json`, or add the record to the target facts catalog before depending on it.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNKNOWN_CAPABILITY_FAMILY,
        default_severity: Severity::Error,
        explanation: "A `targets:` section names a capability family that Hum does not publish in `hum target-facts`.",
        repair: "Use one of the capability families from `hum target-facts --format json`, or add the family to the target facts catalog first.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::UNSUPPORTED_TARGET_DECLARATION,
        default_severity: Severity::Error,
        explanation: "A meaningful `targets:` line does not use a current formal key, so Hum would otherwise ignore a portability promise.",
        repair: "Use `triple:`, `record:`, `target:`, `requires:`, or `denies:` for Milestone 0 target declarations.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::REQUIRED_CAPABILITY_UNAVAILABLE,
        default_severity: Severity::Error,
        explanation: "A `targets:` section requires a capability family that a declared target fact record marks absent or unavailable.",
        repair: "Choose a target that provides the capability, remove the requirement, or add an adapter/profile design before depending on it.",
    },
    DiagnosticInfo {
        code: DiagnosticCode::CONFLICTING_TARGET_CAPABILITY,
        default_severity: Severity::Error,
        explanation: "A `targets:` section both requires and denies the same capability family, making the portability intent contradictory.",
        repair: "Remove one declaration, or split the target policy into separate tasks/profiles with different capability intent.",
    },
];

pub fn all() -> &'static [DiagnosticInfo] {
    DIAGNOSTICS
}

pub fn find(code: &str) -> Option<&'static DiagnosticInfo> {
    let code = code.trim();
    DIAGNOSTICS
        .iter()
        .find(|info| info.code.as_str().eq_ignore_ascii_case(code))
}

#[cfg(test)]
mod tests {
    use super::{all, find};
    use crate::diagnostic::DiagnosticCode;

    #[test]
    fn catalog_contains_known_codes() {
        assert_eq!(
            find("H0201").map(|info| info.code),
            Some(DiagnosticCode::UNDECLARED_SAVE_TARGET)
        );
        assert_eq!(
            find("h0502").map(|info| info.code),
            Some(DiagnosticCode::REGRESSION_MISSING_NOTE)
        );
        assert_eq!(
            find("H0601").map(|info| info.code),
            Some(DiagnosticCode::UNRESOLVED_NAME)
        );
        assert_eq!(
            find("H0604").map(|info| info.code),
            Some(DiagnosticCode::READ_BEFORE_DECLARE)
        );
        assert_eq!(
            find("H0605").map(|info| info.code),
            Some(DiagnosticCode::UNKNOWN_TYPE_NAME)
        );
        assert_eq!(
            find("H0606").map(|info| info.code),
            Some(DiagnosticCode::RETURN_TYPE_MISMATCH)
        );
        assert_eq!(
            find("H0701").map(|info| info.code),
            Some(DiagnosticCode::UNCHECKED_PROSE_CONTRACT)
        );
        assert_eq!(
            find("H0703").map(|info| info.code),
            Some(DiagnosticCode::ENSURES_CONTRACT_VIOLATION)
        );
        assert_eq!(
            find("H0801").map(|info| info.code),
            Some(DiagnosticCode::USE_AFTER_MOVE)
        );
        assert_eq!(
            find("H0802").map(|info| info.code),
            Some(DiagnosticCode::BORROW_PARAMETER_MUTATION)
        );
        assert_eq!(
            find("H0803").map(|info| info.code),
            Some(DiagnosticCode::LINEAR_RESOURCE_NOT_CONSUMED)
        );
        assert_eq!(
            find("H0804").map(|info| info.code),
            Some(DiagnosticCode::LINEAR_RESOURCE_CONSUMED_TWICE)
        );
        assert_eq!(
            find("H0805").map(|info| info.code),
            Some(DiagnosticCode::RETURN_DEPENDENCY_NOT_PARAMETER)
        );
        assert_eq!(
            find("H0807").map(|info| info.code),
            Some(DiagnosticCode::STALE_FIELD_VIEW)
        );
        assert_eq!(
            find("H0808").map(|info| info.code),
            Some(DiagnosticCode::WRITABLE_ALIAS_OVERLAP)
        );
        assert_eq!(
            find("H0809").map(|info| info.code),
            Some(DiagnosticCode::UNSUPPORTED_WRITABLE_ALIAS)
        );
        assert_eq!(
            find("H0907").map(|info| info.code),
            Some(DiagnosticCode::MISSING_FAILURE_DECLARATION)
        );
        assert_eq!(
            find("H1201").map(|info| info.code),
            Some(DiagnosticCode::UNKNOWN_TARGET_FACT_RECORD)
        );
        assert_eq!(
            find("H1204").map(|info| info.code),
            Some(DiagnosticCode::REQUIRED_CAPABILITY_UNAVAILABLE)
        );
        assert_eq!(
            find("H1205").map(|info| info.code),
            Some(DiagnosticCode::CONFLICTING_TARGET_CAPABILITY)
        );
        assert!(find("H9999").is_none());
    }

    #[test]
    fn catalog_has_unique_codes() {
        let mut codes = all()
            .iter()
            .map(|info| info.code.as_str())
            .collect::<Vec<_>>();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), all().len());
    }
}

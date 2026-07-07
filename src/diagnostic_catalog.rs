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
        explanation: "An item is missing a section that Milestone 0 needs in order to trust its shape, such as `why:` or `does:`.",
        repair: "Add the required section with meaningful content.",
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
        explanation: "A task has no `needs:` section, so caller responsibilities and generated precondition tests are missing.",
        repair: "Add `needs:` with the important preconditions, or state that no special preconditions are needed.",
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
        explanation: "A returning task has no `ensures:` section, so its success promises are not visible as graph facts or test obligations.",
        repair: "Add `ensures:` lines that describe what the returned value or changed state must satisfy.",
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
        explanation: "A task has no `cost:` section, so performance expectations are not visible for review.",
        repair: "Add a `cost:` section with time, space, and check expectations appropriate to the task.",
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

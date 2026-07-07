use std::collections::BTreeSet;

use crate::ast::{Item, Section, SourceFile, Task, Test};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Span};
use crate::syntax;

pub fn check_file(file: &SourceFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &file.items {
        check_item(item, &mut diagnostics);
    }
    diagnostics
}

fn check_item(item: &Item, diagnostics: &mut Vec<Diagnostic>) {
    match item {
        Item::App(app) => {
            if app.section("why").is_none() {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::APP_MISSING_WHY,
                    format!("app `{}` has no `why:` section", app.name),
                    Some(app.span.clone()),
                ));
            }
            for item in &app.items {
                check_item(item, diagnostics);
            }
        }
        Item::Type(type_def) => {
            if type_def.fields.is_empty() && type_def.section("keeps").is_none() {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::TYPE_MISSING_SHAPE,
                    format!("type `{}` has no fields or invariant", type_def.name),
                    Some(type_def.span.clone()),
                ));
            }
        }
        Item::Store(store) => {
            if store.ty.is_empty() {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::STORE_MISSING_TYPE,
                    format!("store `{}` does not declare a type", store.name),
                    Some(store.span.clone()),
                ));
            }
            if store.section("why").is_none() && store.section("expects").is_none() {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticCode::STORE_MISSING_PURPOSE,
                    format!("store `{}` has no `why:` or `expects:` section", store.name),
                    Some(store.span.clone()),
                ));
            }
        }
        Item::Task(task) => check_task(task, diagnostics),
        Item::Test(test) => check_test(test, diagnostics),
    }
}

fn check_task(task: &Task, diagnostics: &mut Vec<Diagnostic>) {
    require_section(
        DiagnosticCode::MISSING_REQUIRED_SECTION,
        "task",
        &task.name,
        &task.span,
        task.section("why"),
        "why",
        diagnostics,
    );
    require_section(
        DiagnosticCode::MISSING_REQUIRED_SECTION,
        "task",
        &task.name,
        &task.span,
        task.section("does"),
        "does",
        diagnostics,
    );

    warn_duplicate_sections(&task.sections, diagnostics);
    warn_section_order(
        "task",
        &task.name,
        &task.sections,
        syntax::TASK_SECTION_ORDER,
        diagnostics,
    );

    if task.section("needs").is_none() {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::TASK_MISSING_NEEDS,
            format!("task `{}` has no `needs:` section", task.name),
            Some(task.span.clone()),
        ));
    }
    if task.result.is_some() && task.section("ensures").is_none() {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::TASK_MISSING_ENSURES,
            format!(
                "task `{}` returns a value but has no `ensures:` section",
                task.name
            ),
            Some(task.span.clone()),
        ));
    }

    check_contract_quality(task, diagnostics);
    check_declared_mutation(task, diagnostics);
    check_cost_contract(task, diagnostics);
    check_security_contracts(task, diagnostics);
}

fn check_test(test: &Test, diagnostics: &mut Vec<Diagnostic>) {
    require_section(
        DiagnosticCode::MISSING_REQUIRED_SECTION,
        "test",
        &test.name,
        &test.span,
        test.section("why"),
        "why",
        diagnostics,
    );
    require_section(
        DiagnosticCode::MISSING_REQUIRED_SECTION,
        "test",
        &test.name,
        &test.span,
        test.section("does"),
        "does",
        diagnostics,
    );

    warn_duplicate_sections(&test.sections, diagnostics);
    warn_section_order(
        "test",
        &test.name,
        &test.sections,
        syntax::TEST_SECTION_ORDER,
        diagnostics,
    );

    if test.section("covers").is_none() {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::TEST_MISSING_COVERS,
            format!("test `{}` has no `covers:` section", test.name),
            Some(test.span.clone()),
        ));
    }

    if test
        .modifiers
        .iter()
        .any(|modifier| modifier == "regression")
        && test.section("regression").is_none()
    {
        diagnostics.push(
            Diagnostic::warning(
                DiagnosticCode::REGRESSION_MISSING_NOTE,
                format!("regression test `{}` has no `regression:` note", test.name),
                Some(test.span.clone()),
            )
            .with_help(
                "Record the bug shape so future humans and agents know what must not return.",
            ),
        );
    }
}

fn require_section(
    code: DiagnosticCode,
    kind: &str,
    name: &str,
    span: &Span,
    section: Option<&Section>,
    section_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if section.is_none() {
        diagnostics.push(Diagnostic::error(
            code,
            format!("{kind} `{name}` is missing `{section_name}:`"),
            Some(span.clone()),
        ));
    }
}

fn warn_duplicate_sections(sections: &[Section], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeSet::new();
    for section in sections {
        if !seen.insert(section.name.clone()) {
            diagnostics.push(Diagnostic::warning(
                DiagnosticCode::DUPLICATE_SECTION,
                format!("duplicate `{}` section", section.name),
                Some(section.span.clone()),
            ));
        }
    }
}

fn warn_section_order(
    kind: &str,
    item_name: &str,
    sections: &[Section],
    expected_order: &[&str],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut last_position = None;
    let mut last_section = None;

    for section in sections {
        let Some(position) = expected_order
            .iter()
            .position(|expected| *expected == section.name)
        else {
            continue;
        };

        if let Some(previous_position) = last_position
            && position < previous_position
        {
            let previous = last_section.unwrap_or("an earlier section");
            diagnostics.push(
                Diagnostic::warning(
                    DiagnosticCode::SECTION_OUT_OF_ORDER,
                    format!(
                        "{kind} `{item_name}` places `{}` after `{previous}`",
                        section.name
                    ),
                    Some(section.span.clone()),
                )
                .with_help(format!(
                    "Use the canonical {kind} order: {}.",
                    expected_order.join(", ")
                )),
            );
        }

        if last_position.is_none_or(|previous| position >= previous) {
            last_position = Some(position);
            last_section = Some(section.name.as_str());
        }
    }
}

fn check_contract_quality(task: &Task, diagnostics: &mut Vec<Diagnostic>) {
    for section_name in [
        "needs",
        "ensures",
        "protects",
        "trusts",
        "watch for",
        "allocates",
        "optimizes",
    ] {
        let Some(section) = task.section(section_name) else {
            continue;
        };

        for line in meaningful_lines(section) {
            let Some(reason) = hollow_contract_reason(&line.text) else {
                continue;
            };
            diagnostics.push(
                Diagnostic::warning(
                    DiagnosticCode::HOLLOW_CONTRACT_LINE,
                    format!(
                        "task `{}` has a hollow `{section_name}:` line: {}",
                        task.name, line.text
                    ),
                    Some(line.span.clone()),
                )
                .with_help(format!(
                    "{reason}; write a specific claim that could reject a wrong implementation."
                )),
            );
        }
    }
}

fn hollow_contract_reason(text: &str) -> Option<&'static str> {
    let lower = text.trim().to_ascii_lowercase();
    let compact = lower.split_whitespace().collect::<String>();
    if matches!(
        compact.as_str(),
        "result==result" | "value==value" | "item==item" | "state==state"
    ) {
        return Some("the claim is tautological");
    }

    let normalized = normalize_contract_text(&lower);
    if normalized.is_empty() {
        return None;
    }

    if normalized.starts_with("todo")
        || normalized.starts_with("tbd")
        || normalized.starts_with("fix later")
    {
        return Some("the claim is still a placeholder");
    }

    if matches!(
        normalized.as_str(),
        "true"
            | "always"
            | "always true"
            | "ok"
            | "okay"
            | "works"
            | "it works"
            | "valid"
            | "correct"
            | "safe"
            | "secure"
            | "fast"
            | "optimized"
            | "good"
            | "good enough"
            | "must work"
            | "everything works"
    ) {
        return Some("the claim is too generic to test or prove");
    }

    None
}

fn normalize_contract_text(text: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_space = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_was_space = false;
        } else if !previous_was_space {
            normalized.push(' ');
            previous_was_space = true;
        }
    }
    normalized.trim().to_string()
}

fn check_declared_mutation(task: &Task, diagnostics: &mut Vec<Diagnostic>) {
    let Some(does) = task.section("does") else {
        return;
    };

    let declared_changes = task
        .section("changes")
        .map(section_resource_set)
        .unwrap_or_default();
    let local_mutables = local_mutables(does);

    for line in meaningful_lines(does) {
        if let Some(target) = save_target(&line.text)
            && !declared_changes.contains(&target)
        {
            diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UNDECLARED_SAVE_TARGET,
                    format!(
                        "task `{}` saves into `{target}` without listing it in `changes:`",
                        task.name
                    ),
                    Some(line.span.clone()),
                )
                .with_help(format!(
                    "Add `{target}` under `changes:` or avoid mutating it."
                )),
            );
        }

        if let Some(target) = set_target(&line.text)
            && !declared_changes.contains(&target)
            && !local_mutables.contains(&target)
        {
            diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UNDECLARED_SET_TARGET,
                    format!(
                        "task `{}` sets `{target}` without declaring it mutable",
                        task.name
                    ),
                    Some(line.span.clone()),
                )
                .with_help(format!(
                    "Use `change {target}: Type = ...` for local mutation or list `{target}` under `changes:`."
                )),
            );
        }
    }
}

fn check_cost_contract(task: &Task, diagnostics: &mut Vec<Diagnostic>) {
    let Some(cost) = task.section("cost") else {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::TASK_MISSING_COST,
            format!("task `{}` has no `cost:` section", task.name),
            Some(task.span.clone()),
        ));
        return;
    };

    let check = cost_value(cost, "check");
    let time = cost_value(cost, "time");

    if check.is_none() {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::COST_MISSING_CHECK,
            format!("task `{}` has `cost:` but no `check:` level", task.name),
            Some(cost.span.clone()),
        ));
        return;
    }

    if matches!(check.as_deref(), Some("compile")) && time.is_none() {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::COMPILE_COST_MISSING_TIME,
            format!(
                "task `{}` uses compile-time cost checking without `time:`",
                task.name
            ),
            Some(cost.span.clone()),
        ));
    }

    let Some(does) = task.section("does") else {
        return;
    };

    if matches!(check.as_deref(), Some("compile")) {
        if matches!(time.as_deref(), Some(value) if value == "O(1)") {
            for line in meaningful_lines(does) {
                if line.text.starts_with("for each ") {
                    diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::CONSTANT_COST_HAS_FOR_EACH,
                            format!(
                                "task `{}` claims `time: O(1)` but contains `for each`",
                                task.name
                            ),
                            Some(line.span.clone()),
                        )
                        .with_help("Use a bounded loop, weaken the cost claim, or make the bound explicit enough for Hum to check."),
                    );
                }
            }
        }

        for line in meaningful_lines(does) {
            if line.text.starts_with("while ") && !line.text.chars().any(|ch| ch.is_ascii_digit()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::COMPILE_COST_UNBOUNDED_WHILE,
                        format!(
                            "task `{}` has an unbounded-looking `while` under `check: compile`",
                            task.name
                        ),
                        Some(line.span.clone()),
                    )
                    .with_help("Make the bound visible, such as `while attempts < 16`, and keep an invariant in `keeps:`."),
                );
            }
        }
    }
}

fn check_security_contracts(task: &Task, diagnostics: &mut Vec<Diagnostic>) {
    let uses_security_source = task
        .section("uses")
        .map(|section| {
            meaningful_lines(section).any(|line| {
                line.text.contains("random")
                    || line.text.contains("crypto")
                    || line.text.contains("password")
                    || line.text.contains("token")
            })
        })
        .unwrap_or(false);

    if uses_security_source && task.section("protects").is_none() {
        diagnostics.push(
            Diagnostic::warning(
                DiagnosticCode::SECURITY_MISSING_PROTECTS,
                format!(
                    "task `{}` touches security-sensitive resources but has no `protects:` section",
                    task.name
                ),
                Some(task.span.clone()),
            )
            .with_help("Hum should make the security promise visible at the function boundary."),
        );
    }

    if task.section("trusts").is_some() && task.section("protects").is_none() {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::TRUSTS_MISSING_PROTECTS,
            format!(
                "task `{}` declares `trusts:` without `protects:`",
                task.name
            ),
            Some(task.span.clone()),
        ));
    }
}

fn section_resource_set(section: &Section) -> BTreeSet<String> {
    meaningful_lines(section)
        .filter_map(|line| first_resource(&line.text))
        .collect()
}

fn meaningful_lines(section: &Section) -> impl Iterator<Item = &crate::ast::SectionLine> {
    section.lines.iter().filter(|line| {
        let text = line.text.trim();
        !text.is_empty() && !text.starts_with('#') && !text.starts_with("//")
    })
}

fn first_resource(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    Some(
        text.split_whitespace()
            .next()
            .unwrap_or(text)
            .trim_matches(|ch: char| ch == ',' || ch == '.')
            .to_string(),
    )
}

fn local_mutables(section: &Section) -> BTreeSet<String> {
    meaningful_lines(section)
        .filter_map(|line| line.text.strip_prefix("change "))
        .filter_map(|rest| {
            let target = rest
                .split(|ch: char| ch == ':' || ch == '=' || ch.is_whitespace())
                .find(|part| !part.is_empty())?;
            Some(target.to_string())
        })
        .collect()
}

fn save_target(text: &str) -> Option<String> {
    let rest = text.strip_prefix("save ")?;
    let (_value, target) = rest.rsplit_once(" in ")?;
    first_resource(target)
}

fn set_target(text: &str) -> Option<String> {
    let rest = text.strip_prefix("set ")?.trim();
    rest.split(|ch: char| ch == '=' || ch.is_whitespace())
        .find(|part| !part.is_empty())
        .map(str::to_string)
}

fn cost_value(section: &Section, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    meaningful_lines(section).find_map(|line| {
        line.text
            .strip_prefix(&prefix)
            .map(|value| value.trim().to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::check_file;
    use crate::diagnostic::{DiagnosticCode, Severity};
    use crate::parser::parse_source;

    #[test]
    fn warns_on_hollow_contract_lines() {
        let source = r#"task prove nothing() -> Bool {
  why:
    demonstrate hollow claims
  needs:
    true
  ensures:
    result == result
  protects:
    safe
  cost:
    time: O(1)
    check: warn
  does:
    return true
}
"#;
        let parsed = parse_source("bad.hum", source);
        let diagnostics = check_file(&parsed.file);
        let hollow_count = diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == Severity::Warning
                    && diagnostic.code == DiagnosticCode::HOLLOW_CONTRACT_LINE
            })
            .count();
        assert_eq!(hollow_count, 3, "got diagnostics: {diagnostics:?}");
    }

    #[test]
    fn keeps_specific_contract_lines_quiet() {
        let source = r#"task save session(token: Text) -> Session {
  why:
    keep a security-sensitive session alive
  uses:
    random.secure
  changes:
    sessions
  needs:
    token has at least 128 bits of entropy
  ensures:
    session is saved with a future expiry
  protects:
    expired session cannot authenticate
  trusts:
    operating system isolates process memory
  watch for:
    token collision with an existing session
  cost:
    time: O(1)
    check: warn
  does:
    save session in sessions
    return session
}
"#;
        let parsed = parse_source("good.hum", source);
        let diagnostics = check_file(&parsed.file);
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| { diagnostic.code != DiagnosticCode::HOLLOW_CONTRACT_LINE })
        );
    }

    #[test]
    fn rejects_undeclared_save_target() {
        let source = r#"task save task() {
  why:
    save it
  does:
    save task in tasks
}
"#;
        let parsed = parse_source("bad.hum", source);
        let diagnostics = check_file(&parsed.file);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::UNDECLARED_SAVE_TARGET
                && diagnostic
                    .message
                    .contains("without listing it in `changes:`")
        }));
    }

    #[test]
    fn rejects_constant_cost_claim_with_iteration() {
        let source = r#"task show tasks() {
  why:
    show tasks
  cost:
    time: O(1)
    check: compile
  does:
    for each task in tasks {
      show task
    }
}
"#;
        let parsed = parse_source("bad.hum", source);
        let diagnostics = check_file(&parsed.file);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::CONSTANT_COST_HAS_FOR_EACH
                && diagnostic.message.contains("claims `time: O(1)`")
        }));
    }

    #[test]
    fn warns_when_task_sections_are_out_of_order() {
        let source = r#"task save task() {
  why:
    save it
  does:
    save task in tasks
  needs:
    title is not empty
}
"#;
        let parsed = parse_source("bad.hum", source);
        let diagnostics = check_file(&parsed.file);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Warning
                && diagnostic.code == DiagnosticCode::SECTION_OUT_OF_ORDER
        }));
    }
}

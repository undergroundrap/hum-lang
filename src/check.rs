use std::collections::BTreeSet;

use crate::ast::{Item, Section, SourceFile, Task, Test};
use crate::core_body;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence, Span};
use crate::graph::hollow_contract_reason;
use crate::{syntax, target_facts, writable_field_alias};

#[cfg(test)]
pub fn check_file(file: &SourceFile) -> Vec<Diagnostic> {
    check_file_with_occurrences(file).diagnostics
}

pub(crate) struct CheckOutput {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
}

struct CheckCollector {
    semantic_file_index: usize,
    diagnostics: Vec<Diagnostic>,
    diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet,
}

#[cfg(test)]
pub(crate) fn check_file_with_occurrences(file: &SourceFile) -> CheckOutput {
    check_file_with_semantic_index(file, 0)
}

pub(crate) fn check_file_with_semantic_index(
    file: &SourceFile,
    semantic_file_index: usize,
) -> CheckOutput {
    let mut diagnostics = CheckCollector {
        semantic_file_index,
        diagnostics: Vec::new(),
        diagnostic_occurrences: crate::diagnostic::DiagnosticOccurrenceSet::default(),
    };
    for item in &file.items {
        check_item(item, &mut diagnostics);
    }
    diagnostics
        .diagnostic_occurrences
        .validate()
        .expect("intent and target checks must use registered causes");
    CheckOutput {
        diagnostics: diagnostics.diagnostics,
        diagnostic_occurrences: diagnostics.diagnostic_occurrences,
    }
}

fn check_item(item: &Item, diagnostics: &mut CheckCollector) {
    match item {
        Item::App(app) => {
            check_target_declarations("app", &app.name, &app.sections, diagnostics);
            if app.section("why").is_none() {
                emit(
                    diagnostics,
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(57),
                    "app",
                    Diagnostic::warning(
                        DiagnosticCode::APP_MISSING_WHY,
                        format!("app `{}` has no `why:` section", app.name),
                        Some(app.span.clone()),
                    ),
                );
            }
            for item in &app.items {
                check_item(item, diagnostics);
            }
        }
        Item::Type(type_def) => {
            check_target_declarations("type", &type_def.name, &type_def.sections, diagnostics);
            if type_def.fields.is_empty() && type_def.section("keeps").is_none() {
                emit(
                    diagnostics,
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(58),
                    "type",
                    Diagnostic::warning(
                        DiagnosticCode::TYPE_MISSING_SHAPE,
                        format!("type `{}` has no fields or invariant", type_def.name),
                        Some(type_def.span.clone()),
                    ),
                );
            }
        }
        Item::Store(store) => {
            check_target_declarations("store", &store.name, &store.sections, diagnostics);
            if store.ty.is_empty() {
                emit(
                    diagnostics,
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(59),
                    "store_type",
                    Diagnostic::warning(
                        DiagnosticCode::STORE_MISSING_TYPE,
                        format!("store `{}` does not declare a type", store.name),
                        Some(store.span.clone()),
                    ),
                );
            }
            if store.section("why").is_none() && store.section("expects").is_none() {
                emit(
                    diagnostics,
                    crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(60),
                    "store_purpose",
                    Diagnostic::warning(
                        DiagnosticCode::STORE_MISSING_PURPOSE,
                        format!("store `{}` has no `why:` or `expects:` section", store.name),
                        Some(store.span.clone()),
                    ),
                );
            }
        }
        Item::Task(task) => check_task(task, diagnostics),
        Item::Test(test) => check_test(test, diagnostics),
    }
}

fn check_task(task: &Task, diagnostics: &mut CheckCollector) {
    check_target_declarations("task", &task.name, &task.sections, diagnostics);

    if task.name == "stdout_write" {
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(103), "task_name",
            Diagnostic::error(
                DiagnosticCode::RESERVED_BUILTIN_NAME,
                "task `stdout_write` redeclares Hum's reserved bounded-output built-in",
                Some(task.span.clone()),
            )
            .with_help(
                "Rename this user task; `stdout_write` is reserved for `stdout_write(text: Text) -> Result Unit, OutputError`.",
            ),
        );
    }

    if task.name == "clock_replay_tick" {
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(104), "task_name",
            Diagnostic::error(
                DiagnosticCode::RESERVED_REPLAY_BUILTIN_NAME,
                "task `clock_replay_tick` redeclares Hum's reserved runner-replay built-in",
                Some(task.span.clone()),
            )
            .with_help(
                "Rename this user task; `clock_replay_tick` is reserved for `clock_replay_tick() -> Result UInt, ReplayClockError`.",
            ),
        );
    }

    if task.name == "files_read_text" {
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(105), "task_name",
            Diagnostic::error(
                DiagnosticCode::RESERVED_FILE_READ_BUILTIN_NAME,
                "task `files_read_text` redeclares Hum's reserved hardened file-read built-in",
                Some(task.span.clone()),
            )
            .with_help(
                "Rename this user task; `files_read_text` is reserved for `files_read_text(path: Path) -> Result Text, FileReadError`.",
            ),
        );
    }

    if task.section("why").is_none() && task_missing_why_is_suspicious(task) {
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(61), "task_why",
            Diagnostic::warning(
                DiagnosticCode::MISSING_REQUIRED_SECTION,
                format!("task `{}` is missing `why:` for nontrivial behavior", task.name),
                Some(task.span.clone()),
            )
            .with_help("Add `why:` when effects, failure modes, or body size make the purpose non-obvious."),
        );
    }
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

    if task.section("needs").is_none() && task_missing_needs_is_suspicious(task) {
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(63), "task_needs",
            Diagnostic::warning(
                DiagnosticCode::TASK_MISSING_NEEDS,
                format!("task `{}` has no `needs:` section for a risky boundary", task.name),
                Some(task.span.clone()),
            )
            .with_help("Add `needs:` when callers must satisfy a real precondition; do not add filler for pure local code."),
        );
    }
    if task.result.is_some()
        && task.section("ensures").is_none()
        && task_missing_ensures_is_suspicious(task)
    {
        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(65), "task_ensures",
            Diagnostic::warning(
                DiagnosticCode::TASK_MISSING_ENSURES,
                format!(
                    "task `{}` returns a value across a nontrivial boundary but has no `ensures:` section",
                    task.name
                ),
                Some(task.span.clone()),
            )
            .with_help("Add `ensures:` when the result promise is not obvious from a small pure body."),
        );
    }

    check_contract_quality(task, diagnostics);
    check_declared_mutation(task, diagnostics);
    check_cost_contract(task, diagnostics);
    check_security_contracts(task, diagnostics);
}

fn check_test(test: &Test, diagnostics: &mut CheckCollector) {
    check_target_declarations("test", &test.name, &test.sections, diagnostics);

    if test.section("why").is_none() && test_missing_why_is_suspicious(test) {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(61),
            "test_why",
            Diagnostic::warning(
                DiagnosticCode::MISSING_REQUIRED_SECTION,
                format!(
                    "test `{}` is missing `why:` for nontrivial evidence",
                    test.name
                ),
                Some(test.span.clone()),
            )
            .with_help(
                "Add `why:` when the evidence shape is not obvious from a small focused test.",
            ),
        );
    }
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
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(76),
            "test_covers",
            Diagnostic::warning(
                DiagnosticCode::TEST_MISSING_COVERS,
                format!("test `{}` has no `covers:` section", test.name),
                Some(test.span.clone()),
            ),
        );
    }

    if test
        .modifiers
        .iter()
        .any(|modifier| modifier == "regression")
        && test.section("regression").is_none()
    {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(77),
            "test_regression",
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

fn task_missing_why_is_suspicious(task: &Task) -> bool {
    task_has_external_boundary(task)
        || task_has_failure_modes(task)
        || task_body_line_count(task) > 6
}

fn task_missing_needs_is_suspicious(task: &Task) -> bool {
    task_has_external_boundary(task)
        || task_has_failure_modes(task)
        || task_body_line_count(task) > 8
}

fn task_missing_ensures_is_suspicious(task: &Task) -> bool {
    task_has_external_boundary(task)
        || task_has_failure_modes(task)
        || task_body_line_count(task) > 5
}

fn task_missing_cost_is_suspicious(task: &Task) -> bool {
    task_has_external_boundary(task)
        || task_body_has_prefix(task, "for each ")
        || task_body_has_prefix(task, "while ")
        || task_body_line_count(task) > 5
}

fn test_missing_why_is_suspicious(test: &Test) -> bool {
    test.modifiers.iter().any(|modifier| {
        matches!(
            modifier.as_str(),
            "property" | "fuzz" | "regression" | "integration" | "model"
        )
    }) || test
        .section("does")
        .map(|section| meaningful_lines(section).count() > 3)
        .unwrap_or(false)
}

fn task_has_external_boundary(task: &Task) -> bool {
    task.section("uses")
        .map(|section| meaningful_lines(section).next().is_some())
        .unwrap_or(false)
        || task
            .section("changes")
            .map(|section| meaningful_lines(section).next().is_some())
            .unwrap_or(false)
        || task_body_has_prefix(task, "save ")
}

fn task_has_failure_modes(task: &Task) -> bool {
    task.result
        .as_deref()
        .is_some_and(|result| result.split_whitespace().next() == Some("Result"))
        || task
            .section("fails when")
            .map(|section| meaningful_lines(section).next().is_some())
            .unwrap_or(false)
        || task_body_has_prefix(task, "fail ")
}

fn task_body_line_count(task: &Task) -> usize {
    task.section("does")
        .map(|section| meaningful_lines(section).count())
        .unwrap_or(0)
}

fn task_body_has_prefix(task: &Task, prefix: &str) -> bool {
    task.section("does")
        .map(|section| meaningful_lines(section).any(|line| line.text.starts_with(prefix)))
        .unwrap_or(false)
}

fn check_target_declarations(
    kind: &str,
    item_name: &str,
    sections: &[Section],
    diagnostics: &mut CheckCollector,
) {
    for section in sections.iter().filter(|section| section.name == "targets") {
        let mut target_records = Vec::new();
        let mut required_capability_families = Vec::new();
        let mut denied_capability_families = Vec::new();

        for line in meaningful_lines(section) {
            match target_facts::parse_source_target_declaration_line(&line.text) {
                Some((target_facts::SourceTargetDeclarationKind::TargetFactRecord, value)) => {
                    if target_facts::is_known_target_fact_record(&value) {
                        target_records.push(value);
                    } else {
                        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(118), "target_record",
                            Diagnostic::error(
                                DiagnosticCode::UNKNOWN_TARGET_FACT_RECORD,
                                format!(
                                    "{kind} `{item_name}` names unknown target fact record `{value}`"
                                ),
                                Some(line.span.clone()),
                            )
                            .with_help(
                                "Use a record ID from `hum target-facts --format json` or add a fixture record before depending on it.",
                            ),
                        );
                    }
                }
                Some((target_facts::SourceTargetDeclarationKind::RequiredCapabilityFamily, value)) => {
                    if target_facts::is_known_capability_family(&value) {
                        required_capability_families.push((value, line.span.clone()));
                    } else {
                        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(119), "target_requires",
                            Diagnostic::error(
                                DiagnosticCode::UNKNOWN_CAPABILITY_FAMILY,
                                format!(
                                    "{kind} `{item_name}` names unknown capability family `{value}`"
                                ),
                                Some(line.span.clone()),
                            )
                            .with_help(
                                "Use a capability family from `hum target-facts --format json` or add the family before depending on it.",
                            ),
                        );
                    }
                }
                Some((target_facts::SourceTargetDeclarationKind::DeniedCapabilityFamily, value)) => {
                    if target_facts::is_known_capability_family(&value) {
                        denied_capability_families.push((value, line.span.clone()));
                    } else {
                        emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(119), "target_denies",
                            Diagnostic::error(
                                DiagnosticCode::UNKNOWN_CAPABILITY_FAMILY,
                                format!(
                                    "{kind} `{item_name}` names unknown capability family `{value}`"
                                ),
                                Some(line.span.clone()),
                            )
                            .with_help(
                                "Use a capability family from `hum target-facts --format json` or add the family before depending on it.",
                            ),
                        );
                    }
                }
                None => emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(120), "target_line",
                    Diagnostic::error(
                        DiagnosticCode::UNSUPPORTED_TARGET_DECLARATION,
                        format!(
                            "{kind} `{item_name}` has unsupported `targets:` line: {}",
                            line.text
                        ),
                        Some(line.span.clone()),
                    )
                    .with_help(
                        "Use `triple:`, `record:`, `target:`, `requires:`, or `denies:` in `targets:` for Milestone 0.",
                    ),
                ),
            }
        }

        let required_names = required_capability_families
            .iter()
            .map(|(family, _span)| family.as_str())
            .collect::<BTreeSet<_>>();
        let mut emitted_conflicts = BTreeSet::new();
        for (family, span) in &denied_capability_families {
            if required_names.contains(family.as_str()) && emitted_conflicts.insert(family.clone())
            {
                emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(122), "target_conflict",
                    Diagnostic::error(
                        DiagnosticCode::CONFLICTING_TARGET_CAPABILITY,
                        format!(
                            "{kind} `{item_name}` both requires and denies capability `{family}`"
                        ),
                        Some(span.clone()),
                    )
                    .with_help(
                        "Remove one declaration, or split the policy into separate tasks/profiles with different capability intent.",
                    ),
                );
            }
        }

        let mut emitted_unavailable = BTreeSet::new();
        for target_record in &target_records {
            for (family, span) in &required_capability_families {
                let Some(status) =
                    target_facts::target_required_capability_status(target_record, family)
                else {
                    continue;
                };
                if status.is_unavailable()
                    && emitted_unavailable.insert((
                        status.target_id.to_string(),
                        family.clone(),
                        span.file.clone(),
                        span.line,
                        span.column,
                    ))
                {
                    emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(121), "target_requirement",
                        Diagnostic::error(
                            DiagnosticCode::REQUIRED_CAPABILITY_UNAVAILABLE,
                            format!(
                                "{kind} `{item_name}` requires capability `{family}` but target `{}` reports `{}`",
                                status.target_id, status.availability
                            ),
                            Some(span.clone()),
                        )
                        .with_help(format!(
                            "Target note: {}. Choose a different target, remove the requirement, or add an adapter/profile design before relying on it.",
                            status.note
                        )),
                    );
                }
            }
        }
    }
}
fn require_section(
    code: DiagnosticCode,
    kind: &str,
    name: &str,
    span: &Span,
    section: Option<&Section>,
    section_name: &str,
    diagnostics: &mut CheckCollector,
) {
    if section.is_none() {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(61),
            "required_section",
            Diagnostic::error(
                code,
                format!("{kind} `{name}` is missing `{section_name}:`"),
                Some(span.clone()),
            ),
        );
    }
}

fn warn_duplicate_sections(sections: &[Section], diagnostics: &mut CheckCollector) {
    let mut seen = BTreeSet::new();
    for section in sections {
        if !seen.insert(section.name.clone()) {
            emit(
                diagnostics,
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(62),
                "duplicate_section",
                Diagnostic::warning(
                    DiagnosticCode::DUPLICATE_SECTION,
                    format!("duplicate `{}` section", section.name),
                    Some(section.span.clone()),
                ),
            );
        }
    }
}

fn warn_section_order(
    kind: &str,
    item_name: &str,
    sections: &[Section],
    expected_order: &[&str],
    diagnostics: &mut CheckCollector,
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
            emit(
                diagnostics,
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(64),
                "section_order",
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

fn check_contract_quality(task: &Task, diagnostics: &mut CheckCollector) {
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
            emit(
                diagnostics,
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(66),
                "contract_line",
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

fn check_declared_mutation(task: &Task, diagnostics: &mut CheckCollector) {
    let Some(does) = task.section("does") else {
        return;
    };

    let declared_changes = task
        .section("changes")
        .map(section_resource_set)
        .unwrap_or_default();
    let local_mutables = local_mutables(does);
    let parameter_roots = task
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();

    let body = core_body::analyze_does_section(does);
    for statement in &body.statements {
        if let Some(target) = statement.save_target()
            && !declared_changes.contains(target)
        {
            emit(
                diagnostics,
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(67),
                "save_target",
                Diagnostic::error(
                    DiagnosticCode::UNDECLARED_SAVE_TARGET,
                    format!(
                        "task `{}` saves into `{target}` without listing it in `changes:`",
                        task.name
                    ),
                    Some(statement.span.clone()),
                )
                .with_help(format!(
                    "Add `{target}` under `changes:` or avoid mutating it."
                )),
            );
        }

        if let Some(target) = statement
            .set_target()
            .and_then(|target| target.direct_place().map(|(root, _, _)| root))
            && !declared_changes.contains(target)
            && !local_mutables.contains(target)
            && !parameter_roots.contains(target)
        {
            emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(68), "set_target",
                Diagnostic::error(
                    DiagnosticCode::UNDECLARED_SET_TARGET,
                    format!(
                        "task `{}` sets `{target}` without declaring it mutable",
                        task.name
                    ),
                    Some(statement.span.clone()),
                )
                .with_help(format!(
                    "Use `change {target}: Type = ...` for local mutation or list `{target}` under `changes:`."
                )),
            );
        }
    }
}

fn check_cost_contract(task: &Task, diagnostics: &mut CheckCollector) {
    let Some(cost) = task.section("cost") else {
        if task_missing_cost_is_suspicious(task) {
            emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(69), "task_cost",
                Diagnostic::warning(
                    DiagnosticCode::TASK_MISSING_COST,
                    format!("task `{}` has no `cost:` section for nontrivial work", task.name),
                    Some(task.span.clone()),
                )
                .with_help("Add `cost:` when loops, external effects, or larger bodies make resource behavior worth reviewing."),
            );
        }
        return;
    };

    let check = cost_value(cost, "check");
    let time = cost_value(cost, "time");

    if check.is_none() {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(70),
            "cost_check",
            Diagnostic::warning(
                DiagnosticCode::COST_MISSING_CHECK,
                format!("task `{}` has `cost:` but no `check:` level", task.name),
                Some(cost.span.clone()),
            ),
        );
        return;
    }

    if matches!(check.as_deref(), Some("compile")) && time.is_none() {
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(71),
            "cost_time",
            Diagnostic::error(
                DiagnosticCode::COMPILE_COST_MISSING_TIME,
                format!(
                    "task `{}` uses compile-time cost checking without `time:`",
                    task.name
                ),
                Some(cost.span.clone()),
            ),
        );
    }

    let Some(does) = task.section("does") else {
        return;
    };

    if matches!(check.as_deref(), Some("compile")) {
        if matches!(time.as_deref(), Some(value) if value == "O(1)") {
            let body = core_body::analyze_does_section(does);
            for statement in &body.statements {
                if statement.kind == "for_each_header" {
                    emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(72), "cost_loop",
                        Diagnostic::error(
                            DiagnosticCode::CONSTANT_COST_HAS_FOR_EACH,
                            format!(
                                "task `{}` claims `time: O(1)` but contains `for each`",
                                task.name
                            ),
                            Some(statement.span.clone()),
                        )
                        .with_help("Use a bounded loop, weaken the cost claim, or make the bound explicit enough for Hum to check."),
                    );
                }
            }
        }

        let body = core_body::analyze_does_section(does);
        for statement in &body.statements {
            if statement.kind == "while_header"
                && !statement
                    .condition()
                    .is_some_and(canonical_contains_integer)
            {
                emit(diagnostics, crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(73), "cost_while",
                    Diagnostic::error(
                        DiagnosticCode::COMPILE_COST_UNBOUNDED_WHILE,
                        format!(
                            "task `{}` has an unbounded-looking `while` under `check: compile`",
                            task.name
                        ),
                        Some(statement.span.clone()),
                    )
                    .with_help("Make the bound visible, such as `while attempts < 16`, and keep an invariant in `keeps:`."),
                );
            }
        }
    }
}

fn check_security_contracts(task: &Task, diagnostics: &mut CheckCollector) {
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
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(74),
            "security_protects",
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
        emit(
            diagnostics,
            crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(75),
            "security_trusts",
            Diagnostic::warning(
                DiagnosticCode::TRUSTS_MISSING_PROTECTS,
                format!(
                    "task `{}` declares `trusts:` without `protects:`",
                    task.name
                ),
                Some(task.span.clone()),
            ),
        );
    }
}

fn emit(
    diagnostics: &mut CheckCollector,
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    node_role: &'static str,
    diagnostic: Diagnostic,
) {
    let event = diagnostics.diagnostics.len();
    let semantic_origin = format!(
        "check-node:file-{file_index}:event-{event}:role-{node_role}",
        file_index = diagnostics.semantic_file_index,
    );
    let route = vec![
        format!("check_file_index={}", diagnostics.semantic_file_index),
        format!("check_event={event}"),
        format!("check_node_role={node_role}"),
    ];
    let (diagnostic, occurrence) =
        DiagnosticOccurrence::producer_diagnostic(cause_key, diagnostic, semantic_origin, route)
            .expect("check diagnostic cause and producer identity must be registered");
    diagnostics
        .diagnostic_occurrences
        .insert_owned(occurrence)
        .expect("check diagnostic occurrences must be unique");
    diagnostics.diagnostics.push(diagnostic);
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
    let token = text
        .split_whitespace()
        .next()
        .unwrap_or(text)
        .trim_matches(|ch: char| ch == ',' || ch == '.');
    token
        .split(['.', '['])
        .next()
        .filter(|root| !root.is_empty())
        .map(str::to_string)
}

fn local_mutables(section: &Section) -> BTreeSet<String> {
    let body = core_body::analyze_does_section(section);
    let mut mutables = body
        .statements
        .iter()
        .filter(|statement| statement.binding_is_mutable())
        .filter_map(|statement| statement.binding_name().map(str::to_string))
        .collect::<BTreeSet<_>>();
    mutables.extend(
        body.statements
            .iter()
            .filter_map(writable_field_alias::candidate_name),
    );
    mutables
}

fn canonical_contains_integer(expression: &crate::ast::CanonicalExpression) -> bool {
    use crate::ast::CanonicalExpressionKind as Kind;
    match &expression.kind {
        Kind::UIntLiteral(_) | Kind::IntLiteral(_) => true,
        Kind::Field { base, .. }
        | Kind::Element { base, .. }
        | Kind::Group(base)
        | Kind::Permission { value: base, .. } => canonical_contains_integer(base),
        Kind::Try { call, .. } => canonical_contains_integer(call),
        Kind::ListLiteral(values) => values.iter().any(canonical_contains_integer),
        Kind::RecordLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| canonical_contains_integer(value)),
        Kind::Call { callee, arguments } => {
            canonical_contains_integer(callee) || arguments.iter().any(canonical_contains_integer)
        }
        Kind::Binary { left, right, .. } => {
            canonical_contains_integer(left) || canonical_contains_integer(right)
        }
        Kind::Unit
        | Kind::Identifier(_)
        | Kind::BoolLiteral(_)
        | Kind::TextLiteral(_)
        | Kind::Unsupported => false,
    }
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
    fn producer_identity_is_unique_across_files_with_matching_check_events() {
        let source = "type Empty {\n}\n";
        let first = parse_source("first.hum", source);
        let second = parse_source("second.hum", source);
        let first_checked = super::check_file_with_semantic_index(&first.file, 0);
        let second_checked = super::check_file_with_semantic_index(&second.file, 1);
        let mut combined = first_checked.diagnostic_occurrences;
        combined
            .extend_owned(&second_checked.diagnostic_occurrences)
            .expect("file-owned check occurrences must remain distinct");
        assert_eq!(combined.occurrences().count(), 2);
    }

    #[test]
    fn check_occurrence_identity_does_not_depend_on_display_path() {
        let source = "type Empty {\n}\n";
        let first = parse_source("first-display.hum", source);
        let renamed = parse_source("renamed-display.hum", source);
        let first = super::check_file_with_semantic_index(&first.file, 7);
        let renamed = super::check_file_with_semantic_index(&renamed.file, 7);
        let first = first
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("check occurrence");
        let renamed = renamed
            .diagnostic_occurrences
            .occurrences()
            .next()
            .expect("renamed check occurrence");
        assert_eq!(first.semantic_origin(), renamed.semantic_origin());
        assert_eq!(first.relationship_route(), renamed.relationship_route());
        assert!(!first.semantic_origin().contains(".hum"));
    }

    #[test]
    fn rejects_unknown_target_declarations() {
        let source = r#"task bad_target() {
  why:
    show target validation
  targets:
    triple: mars32-secret
    requires: os.telepathy
    maybe: os.network
  needs:
    manifest exists
  cost:
    time: O(1)
    check: warn
  does:
    return manifest
}
"#;
        let parsed = parse_source("bad-targets.hum", source);
        let diagnostics = check_file(&parsed.file);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::UNKNOWN_TARGET_FACT_RECORD
                && diagnostic
                    .message
                    .contains("unknown target fact record `mars32-secret`")
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::UNKNOWN_CAPABILITY_FAMILY
                && diagnostic
                    .message
                    .contains("unknown capability family `os.telepathy`")
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::UNSUPPORTED_TARGET_DECLARATION
                && diagnostic.message.contains("unsupported `targets:` line")
        }));
    }
    #[test]
    fn rejects_conflicting_target_capability_intent() {
        let source = r#"task confused_target() {
  why:
    show contradictory target policy
  targets:
    triple: windows-x86_64-msvc
    requires: os.network
    denies: os.network
  needs:
    endpoint is declared
  cost:
    time: O(1)
    check: warn
  does:
    return endpoint
}
"#;
        let parsed = parse_source("confused-target.hum", source);
        let diagnostics = check_file(&parsed.file);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::CONFLICTING_TARGET_CAPABILITY
                && diagnostic
                    .message
                    .contains("both requires and denies capability `os.network`")
        }));
    }

    #[test]
    fn rejects_required_capability_unavailable_on_declared_target() {
        let source = r#"task bad_target() {
  why:
    show target capability matching
  targets:
    triple: wasm32-wasi-preview1
    requires: os.network
  needs:
    manifest exists
  cost:
    time: O(1)
    check: warn
  does:
    return manifest
}
"#;
        let parsed = parse_source("bad-target-capability.hum", source);
        let diagnostics = check_file(&parsed.file);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == Severity::Error
                && diagnostic.code == DiagnosticCode::REQUIRED_CAPABILITY_UNAVAILABLE
                && diagnostic
                    .message
                    .contains("requires capability `os.network`")
                && diagnostic
                    .message
                    .contains("target `wasm32-wasi-preview1` reports `absent_by_default`")
        }));
    }

    #[test]
    fn warns_on_hollow_contract_lines() {
        let source = r#"task prove_nothing() -> Bool {
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
        let source = r#"task save_session(token: Text) -> Session {
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
        let source = r#"task save_task() {
  why:
    save it
  does:
    save item in tasks
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
        let source = r#"task show_tasks() {
  why:
    show_tasks
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
        let source = r#"task save_task() {
  why:
    save it
  does:
    save item in tasks
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

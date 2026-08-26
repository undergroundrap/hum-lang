use crate::app_entry::CanonicalNativeLayout;
use crate::ast::Program;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOccurrence};

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const INTEGER_SIGN_FEATURE: &str = "canonical_integer_sign_app_v0";
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const CONSTANT_TEXT_OUTPUT_FEATURE: &str = "canonical_constant_text_output_app_v0";

#[derive(Debug)]
pub(crate) enum NativeProgramFeature {
    IntegerSign(crate::type_check::CanonicalIntegerSignTypeAuthority),
    ConstantTextOutput(crate::type_check::CanonicalConstantTextTypeAuthority),
}

impl NativeProgramFeature {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn id(&self) -> &'static str {
        match self {
            Self::IntegerSign(_) => INTEGER_SIGN_FEATURE,
            Self::ConstantTextOutput(_) => CONSTANT_TEXT_OUTPUT_FEATURE,
        }
    }
}

pub(crate) struct NativeProgramAnalysis {
    pub(crate) feature: Option<NativeProgramFeature>,
    pub(crate) diagnostic: Option<Diagnostic>,
    pub(crate) diagnostic_occurrence: Option<DiagnosticOccurrence>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) reason: Option<&'static str>,
}

pub(crate) fn analyze(
    program: &Program,
    diagnostics: &[Diagnostic],
    layout: &CanonicalNativeLayout<'_>,
) -> NativeProgramAnalysis {
    let integer_sign =
        crate::type_check::canonical_integer_sign_type_authority(program, layout, diagnostics)
            .filter(|authority| {
                crate::profile_check::with_profile_for_ir_readiness(
                    program,
                    diagnostics,
                    |profile| {
                        profile
                            .canonical_integer_sign_for(layout, authority)
                            .is_some()
                    },
                )
            });
    let constant_text =
        crate::type_check::canonical_constant_text_type_authority(program, layout, diagnostics)
            .filter(|authority| {
                crate::profile_check::with_profile_for_ir_readiness(
                    program,
                    diagnostics,
                    |profile| {
                        profile
                            .canonical_constant_text_for(layout, authority)
                            .is_some()
                    },
                )
            });

    match (integer_sign, constant_text) {
        (Some(authority), None) => accepted(NativeProgramFeature::IntegerSign(authority)),
        (None, Some(authority)) => accepted(NativeProgramFeature::ConstantTextOutput(authority)),
        (None, None) => rejected(
            181,
            "native_feature_not_supported_v0",
            layout,
            "native execution does not support the program's authenticated typed feature",
            "Use one currently supported typed native feature; native rejection never falls back to the interpreter.",
        ),
        (Some(_), Some(_)) => rejected(
            182,
            "native_feature_ambiguous_v0",
            layout,
            "native execution found more than one authenticated typed feature",
            "Make the native feature structurally unambiguous; no filename, app name, or literal selects a feature.",
        ),
    }
}

fn accepted(feature: NativeProgramFeature) -> NativeProgramAnalysis {
    NativeProgramAnalysis {
        feature: Some(feature),
        diagnostic: None,
        diagnostic_occurrence: None,
        reason: None,
    }
}

fn rejected(
    cause_ordinal: u16,
    reason: &'static str,
    layout: &CanonicalNativeLayout<'_>,
    message: &str,
    help: &str,
) -> NativeProgramAnalysis {
    let diagnostic = Diagnostic::error(
        DiagnosticCode::UNSUPPORTED_NATIVE_PROGRAM_FEATURE,
        message,
        Some(layout.entry.span.clone()),
    )
    .with_related_span("canonical native app", layout.app.span.clone())
    .with_help(help);
    let (diagnostic, diagnostic_occurrence) = DiagnosticOccurrence::producer_diagnostic(
        crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(cause_ordinal),
        diagnostic,
        format!("native-feature-admission:{reason}"),
        vec![format!("native_feature_reason={reason}")],
    )
    .expect("native feature cause must be producer-owned");
    NativeProgramAnalysis {
        feature: None,
        diagnostic: Some(diagnostic),
        diagnostic_occurrence: Some(diagnostic_occurrence),
        reason: Some(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::{CONSTANT_TEXT_OUTPUT_FEATURE, INTEGER_SIGN_FEATURE, NativeProgramFeature};

    struct RecordingOutput(Vec<u8>);

    impl crate::run::OutputAdapter for RecordingOutput {
        fn write(&mut self, bytes: &[u8]) -> Result<(), crate::run::OutputAdapterError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }

    fn analysis(path: &str, source: &str) -> super::NativeProgramAnalysis {
        let parsed = crate::parser::parse_source(path, source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        let checked = crate::check::check_file(&parsed);
        assert!(
            checked
                .iter()
                .all(|diagnostic| diagnostic.severity != crate::diagnostic::Severity::Error),
            "{checked:#?}"
        );
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let app = crate::app_entry::analyze(&program);
        let layout =
            crate::app_entry::analyze_canonical_native_layout(&program, path, app.entry.as_ref())
                .layout
                .expect("canonical layout");
        super::analyze(&program, &[], &layout)
    }

    fn feature(path: &str, source: &str) -> NativeProgramFeature {
        analysis(path, source)
            .feature
            .expect("typed native feature")
    }

    #[test]
    fn native_feature_discrimination_is_typed_and_load_bearing() {
        assert_eq!(
            CONSTANT_TEXT_OUTPUT_FEATURE,
            concat!("canonical_constant_text_output_app_", "v0")
        );
        let integer = feature(
            "programs/integer_sign.hum",
            include_str!("../programs/integer_sign.hum"),
        );
        let text = feature(
            "programs/hello_world.hum",
            include_str!("../programs/hello_world.hum"),
        );
        assert_eq!(integer.id(), INTEGER_SIGN_FEATURE);
        assert_eq!(text.id(), CONSTANT_TEXT_OUTPUT_FEATURE);
        assert!(matches!(integer, NativeProgramFeature::IntegerSign(_)));
        assert!(matches!(text, NativeProgramFeature::ConstantTextOutput(_)));

        let foreign_identity = include_str!("../programs/hello_world.hum")
            .replace("module programs.hello_world", "module programs.foreign")
            .replace("app hello_world", "app foreign");
        assert!(
            matches!(
                analysis("programs/foreign.hum", &foreign_identity).feature,
                Some(NativeProgramFeature::ConstantTextOutput(_))
            ),
            "layout-valid foreign identity reached the wrong typed-feature assertion"
        );

        let foreign = include_str!("../programs/hello_world.hum")
            .replace("Hello, world!", "typed facts, not literal dispatch");
        assert_eq!(
            feature("programs/hello_world.hum", &foreign).id(),
            CONSTANT_TEXT_OUTPUT_FEATURE
        );
        for fixture in [
            include_str!("../fixtures/programs/hello_world/unsupported_helper_call_fail.hum"),
            include_str!("../fixtures/programs/hello_world/unsupported_nonliteral_output_fail.hum"),
            include_str!("../fixtures/programs/hello_world/unsupported_two_writes_fail.hum"),
        ] {
            let parsed = crate::parser::parse_source("programs/foreign.hum", fixture);
            let program = crate::ast::Program {
                files: vec![parsed.file],
            };
            let app = crate::app_entry::analyze(&program);
            let layout = crate::app_entry::analyze_canonical_native_layout(
                &program,
                "programs/foreign.hum",
                app.entry.as_ref(),
            )
            .layout
            .expect("layout-valid unsupported fixture");
            let analysis = super::analyze(&program, &parsed.diagnostics, &layout);
            assert!(analysis.feature.is_none());
            assert_eq!(analysis.reason, Some("native_feature_not_supported_v0"));
            assert_eq!(
                analysis
                    .diagnostic
                    .as_ref()
                    .map(|value| value.code.as_str()),
                Some("H0635")
            );
        }

        let fallback_source =
            include_str!("../fixtures/programs/hello_world/unsupported_nonliteral_output_fail.hum");
        let parsed = crate::parser::parse_source("programs/foreign.hum", fallback_source);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let mut policy = crate::operator_grant::OperatorGrantPolicy::default();
        policy.allow("stdout.write").expect("output authority");
        let mut output = RecordingOutput(Vec::new());
        if !crate::native_admission_requested(true) {
            let _ = crate::run::run_program_with_output(&program, None, &[], &policy, &mut output);
        }
        assert!(
            output.0.is_empty(),
            "native H0635 rejection reached forbidden interpreter fallback output"
        );
    }
}

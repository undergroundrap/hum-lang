#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EligibilityObservation {
    pub case_id: &'static str,
    pub relationship: &'static str,
    pub status: &'static str,
    pub reason: &'static str,
}

pub const PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS: [EligibilityObservation; 5] = [
    EligibilityObservation {
        case_id: "effect.callback_registry",
        relationship: "stored_callable",
        status: "ineligible_without_restructuring",
        reason: "the required callable must be stored beyond its defining call",
    },
    EligibilityObservation {
        case_id: "effect.event_handler_factory",
        relationship: "returned_callable",
        status: "ineligible_without_restructuring",
        reason: "the required callable must be returned to its caller",
    },
    EligibilityObservation {
        case_id: "effect.memoizing_wrapper",
        relationship: "returned_callable",
        status: "ineligible_without_restructuring",
        reason: "the memoizer must return a callable retaining cache state",
    },
    EligibilityObservation {
        case_id: "effect.logging_middleware",
        relationship: "returned_callable",
        status: "ineligible_without_restructuring",
        reason: "an around-call substitute is not the required returned wrapper",
    },
    EligibilityObservation {
        case_id: "effect.linear_capture",
        relationship: "returned_or_stored_callable",
        status: "ineligible_without_restructuring",
        reason: "the required resource-bearing callable must be retained beyond its defining call",
    },
];

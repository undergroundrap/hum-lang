#![deny(unsafe_op_in_unsafe_fn)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriveRoot {
    letter: u16,
    root: [u16; 4],
    device: [u16; 3],
}

impl DriveRoot {
    pub fn from_ascii_letter(letter: u8) -> Option<Self> {
        letter.is_ascii_alphabetic().then(|| {
            let letter = u16::from(letter.to_ascii_uppercase());
            Self {
                letter,
                root: [letter, u16::from(b':'), u16::from(b'\\'), 0],
                device: [letter, u16::from(b':'), 0],
            }
        })
    }

    #[cfg(windows)]
    fn volume_device(self) -> [u16; 7] {
        [
            u16::from(b'\\'),
            u16::from(b'\\'),
            u16::from(b'.'),
            u16::from(b'\\'),
            self.letter,
            u16::from(b':'),
            0,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveLocality {
    /// Defined (decision 0015's `proved` vocabulary) but unreachable from
    /// the live classifier: the grant-first demotion (decision 0029 §14,
    /// WO29 Slice A) demoted the only positive admission to `Unproven`,
    /// mirroring Linux's `Proven`.
    FixedLocal,
    /// Grant-first verdict: the recorded observations are plausible-local
    /// but insufficient to admit; the named reason lives in
    /// `ClassifiedDrive::unproven_reason`.
    Unproven,
    Remote,
    Substituted,
    Removable,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedDrive {
    pub locality: DriveLocality,
    /// Sorted, deduplicated physical disk numbers backing this drive, taken
    /// from the same completely-observed disk list that feeds the locality
    /// classification (the `required_disks` source). Non-empty on every
    /// `Unproven` path that observed the complete extent topology — the
    /// grant-first path, the cause-specific matrix (removable media,
    /// unsupported bus), and the unreadable-removable-status path; empty
    /// on every closed path (Remote, Substituted, Removable, Unknown,
    /// Unsupported).
    ///
    /// These numbers are *backing-device* identity only, never *file*
    /// identity and never *admission*: they name the physical disks behind
    /// the volume as observed, not a proof that the backing is local. A
    /// file's identity comes from `opened_file_identity`, never from these
    /// numbers.
    pub backing_device_identity: Vec<u32>,
    /// Observed-fact lines recorded on every `Unproven` path: the
    /// dependency-walk observation, each disk extent, each required
    /// disk's bus type (numeric value plus name) and removable-media
    /// flag with the query that produced them, the before/after
    /// observation and equality verdict, the configured observed-bus
    /// list, the one-line P1–P4 mapping, and the classification with its
    /// provenance. Empty on every closed path except the known
    /// before/after contradiction path, which preserves the before/after
    /// facts so the contradiction stays visible to the runtime binder.
    /// Unavailable facts are recorded as unavailable — never silently
    /// dropped. Facts are observations only; they never admit.
    pub observed_facts: Vec<String>,
    /// Named reason for the `Unproven` verdict: the insufficient-evidence
    /// reason for the demoted grant-first path, or the cause-specific
    /// reason for the observed-but-unproven matrix (removable media,
    /// unsupported bus, unreadable removable status). `None` unless
    /// `locality == DriveLocality::Unproven`.
    pub unproven_reason: Option<&'static str>,
    /// Full 64-bit volume serial from `FILE_ID_INFO` on the classified
    /// volume's device handle — the same unit as the opened file's
    /// identity serial. `None` on any failure (including the non-Windows
    /// stub). Never truncated, never zero-extended: the consumer binds
    /// this against `FILE_ID_INFO` serials with direct equality.
    pub volume_serial: Option<u64>,
    /// Known before/after contradiction: the classifier observed the
    /// drive's preliminary observation change (or stop being a
    /// candidate) mid-inspection. Internal to the
    /// producer→reducer→binder path — never admission policy, never
    /// public schema. The runtime external-trust binder rejects a
    /// contradicted bundle before payload consumption even when every
    /// other comparison would agree: matching trust covers missing
    /// evidence, never contradictory evidence.
    pub contradiction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum DriveTypeObservation {
    Unknown,
    MissingRoot,
    Removable,
    Fixed,
    Remote,
    Optical,
    RamDisk,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum QueryState<T> {
    Complete(T),
    ApiFailure,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct PreliminaryObservation {
    drive_type: DriveTypeObservation,
    mapping: QueryState<Vec<u16>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum DependencyObservation {
    None,
    #[cfg(windows)]
    Present,
    #[cfg(test)]
    Vhd,
    #[cfg(test)]
    Vhdx,
    #[cfg(test)]
    Iso,
    #[cfg(test)]
    UncHostedVhd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct ExtentObservation {
    disk_number: u32,
    starting_offset: i64,
    extent_length: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct DiskObservation {
    disk_number: u32,
    removable: bool,
    bus_type: u32,
}

/// The per-disk outcome of the descriptor-query walk (WO29 Item 2):
/// the producer records one outcome per required disk so the reducer can
/// distinguish an attempted-but-failed query from an attempted-but-
/// undecodable one from a query that never ran — instead of collapsing
/// the whole walk into a single `ApiFailure` that claims every
/// descriptor query failed when earlier queries succeeded or later
/// queries never ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum DiskQueryOutcome {
    /// The descriptor query ran and returned a decodable observation.
    Observed(DiskObservation),
    /// The device opened, but the descriptor query (IOCTL or response
    /// validation before decode) failed. Distinct from `OpenFailed`:
    /// this claims the query RAN and failed — the evidence must never
    /// make that claim for a device that never opened.
    Failed,
    /// The descriptor query ran but the returned data failed validation
    /// (undecodable) — the determination is unavailable, not failed.
    Undecodable,
    /// The device could not be opened, so no descriptor query ran. A
    /// failed CloseHandle is NOT this outcome: cleanup failure is
    /// recorded separately and a successful observation is preserved.
    OpenFailed,
    /// The descriptor query never ran: an earlier disk's query aborted
    /// the producer's walk.
    NotAttempted,
}

/// One required disk's descriptor-query outcome, in required-disk order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct DiskQueryRecord {
    disk_number: u32,
    outcome: DiskQueryOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct InspectionEvidence {
    before: PreliminaryObservation,
    dependency: QueryState<DependencyObservation>,
    extents: QueryState<Vec<ExtentObservation>>,
    disks: QueryState<Vec<DiskQueryRecord>>,
    closes: QueryState<()>,
    after: PreliminaryObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(any(windows, test))]
enum PreliminaryClass {
    Candidate,
    Closed(DriveLocality),
}

#[cfg(any(windows, test))]
fn classify_preliminary(observation: &PreliminaryObservation) -> PreliminaryClass {
    let target = match &observation.mapping {
        QueryState::Complete(target) if !target.is_empty() => target.as_slice(),
        QueryState::Complete(_) | QueryState::ApiFailure | QueryState::Partial => {
            return PreliminaryClass::Closed(DriveLocality::Unknown);
        }
    };

    if starts_ascii_case_insensitive(target, r"\Device\Mup")
        || starts_ascii_case_insensitive(target, r"\Device\LanmanRedirector")
    {
        return PreliminaryClass::Closed(DriveLocality::Remote);
    }
    if starts_ascii_case_insensitive(target, r"\??\")
        || starts_ascii_case_insensitive(target, r"\DosDevices\")
    {
        return PreliminaryClass::Closed(DriveLocality::Substituted);
    }

    match observation.drive_type {
        DriveTypeObservation::Remote => PreliminaryClass::Closed(DriveLocality::Remote),
        DriveTypeObservation::Removable
        | DriveTypeObservation::Optical
        | DriveTypeObservation::RamDisk => PreliminaryClass::Closed(DriveLocality::Removable),
        DriveTypeObservation::Fixed if is_ordinary_fixed_target(target) => {
            PreliminaryClass::Candidate
        }
        DriveTypeObservation::Unknown
        | DriveTypeObservation::MissingRoot
        | DriveTypeObservation::Fixed
        | DriveTypeObservation::Other => PreliminaryClass::Closed(DriveLocality::Unknown),
    }
}

/// Named reason for the demoted grant-first `Unproven` verdict: the
/// observed-bus observations are plausible-local, but guest-visible
/// bus-type evidence cannot exclude invisible (hypervisor-interposed)
/// backing, so the evidence is insufficient for admission. Exact string is
/// the builder's choice (decision 0029 §14, reviewed).
#[cfg(any(windows, test))]
pub const REASON_INSUFFICIENT_EVIDENCE: &str = "windows_locality_unproven_insufficient_evidence_v0";

/// Named reason for the observed-but-unproven removable-media verdict
/// (WO29 Item 2): the backing-disk observations were completely recorded,
/// but a required disk reports `RemovableMedia` set. Removable backing —
/// SD/MMC cards, USB readers, welded media reported removable — cannot be
/// admitted, but the observation is named, not a reasonless `Unknown`.
/// Exact string is the builder's choice (decision 0029 §14, reviewed).
#[cfg(any(windows, test))]
pub const REASON_REMOVABLE_MEDIA: &str = "windows_locality_unproven_removable_media_v0";

/// Named reason for the observed-but-unproven unsupported-bus verdict
/// (WO29 Item 2): a required disk's bus type is not on the observed
/// local-bus list, so no local-bus fact can be recorded for that disk.
/// The numeric bus value is still carried in the observed facts as an
/// observation — never dropped. Exact string is the builder's choice
/// (decision 0029 §14, reviewed).
#[cfg(any(windows, test))]
pub const REASON_UNSUPPORTED_BUS: &str = "windows_locality_unproven_unsupported_bus_v0";

/// Named reason for the observed-but-unproven unreadable-removable-status
/// verdict (WO29 Item 2): the extent topology was completely observed,
/// but the removable-media determination (the
/// `STORAGE_DEVICE_DESCRIPTOR` query) failed, so removable status is
/// unavailable. The unavailable facts are recorded as unavailable in the
/// observed facts — never a reasonless `Unknown`. Exact string is the
/// builder's choice (decision 0029 §14, reviewed).
#[cfg(any(windows, test))]
pub const REASON_REMOVABLE_STATUS_UNREADABLE: &str =
    "windows_locality_unproven_removable_status_unreadable_v0";

/// Test-only projection of the classification verdict: the bundled tests
/// exercise the evidence-to-verdict mapping without opening real devices.
/// (Previously also compiled on Windows; narrowed after the cross-target
/// build flagged it as dead code there.)
#[cfg(test)]
fn classify_evidence(evidence: &InspectionEvidence) -> DriveLocality {
    classify_evidence_detail(evidence).locality
}

/// The full classification verdict for one complete inspection: locality,
/// the sorted/deduplicated backing-disk numbers kept as device observation
/// (not admission), the observed-fact lines, and the named `Unproven`
/// reason (`None` unless the verdict is `Unproven`).
///
/// The classification logic matches `classify_evidence`; this widens the
/// return so the Windows `classify` entry point can thread observations
/// through to `ClassifiedDrive`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(any(windows, test))]
struct EvidenceVerdict {
    locality: DriveLocality,
    backing_device_identity: Vec<u32>,
    observed_facts: Vec<String>,
    unproven_reason: Option<&'static str>,
    /// Known before/after contradiction: the classifier observed the
    /// drive's preliminary observation change (or stop being a
    /// candidate) mid-inspection. Contradictory evidence is not
    /// missing evidence — the runtime external-trust binder must reject
    /// it before payload consumption, never cover it with matching
    /// trust. Internal to the verdict-to-binder path; not admission
    /// policy, not public schema.
    contradiction: bool,
}

#[cfg(any(windows, test))]
impl EvidenceVerdict {
    /// A closed (non-candidate) verdict carries no disk identity, no
    /// observed facts, and no `Unproven` reason.
    fn closed(locality: DriveLocality) -> Self {
        debug_assert_ne!(
            locality,
            DriveLocality::Unproven,
            "every Unproven path is observed and named — see observed_unproven"
        );
        Self {
            locality,
            backing_device_identity: Vec::new(),
            observed_facts: Vec::new(),
            unproven_reason: None,
            contradiction: false,
        }
    }

    /// A known before/after contradiction: the verdict stays closed
    /// (nothing admits), but the before/after facts are preserved and
    /// the contradiction marker is set so the runtime external-trust
    /// binder can refuse it before payload consumption. Matching trust
    /// must never reinterpret a known contradiction as acceptable
    /// absence.
    fn closed_contradiction(
        before: &PreliminaryObservation,
        after: &PreliminaryObservation,
    ) -> Self {
        let equality = if before == after { "match" } else { "mismatch" };
        Self {
            locality: DriveLocality::Unknown,
            backing_device_identity: Vec::new(),
            observed_facts: vec![
                format!(
                    "before: {}; after: {}; equality: {}",
                    preliminary_summary(before),
                    preliminary_summary(after),
                    equality
                ),
                "classification: Unknown (known before/after contradiction; matching trust must not cover contradictory evidence)"
                    .to_string(),
            ],
            unproven_reason: None,
            contradiction: true,
        }
    }

    /// An observed-but-unproven verdict: the extent topology was
    /// completely observed, so the verdict keeps the observed disk
    /// numbers (device observation, not admission), the complete
    /// observed-fact record, and the named `Unproven` reason — the
    /// grant-first insufficient-evidence reason or one of the
    /// cause-specific reasons (WO29 Item 2).
    fn observed_unproven(
        reason: &'static str,
        backing_device_identity: Vec<u32>,
        observed_facts: Vec<String>,
    ) -> Self {
        Self {
            locality: DriveLocality::Unproven,
            backing_device_identity,
            observed_facts,
            unproven_reason: Some(reason),
            contradiction: false,
        }
    }
}

/// The observed local-bus list (WO29 Item 2): which bus types yield
/// local-bus observed facts. Observation only — no bus type on this list
/// independently earns `proved` (decision 0029 §14). The list is emitted
/// into the evidence bundle so the next widening is visibly a decision.
#[cfg(any(windows, test))]
const OBSERVED_BUS_TYPES: &[(u32, &str)] = &[
    (BUS_TYPE_ATA, "ATA"),
    (BUS_TYPE_SATA, "SATA"),
    (BUS_TYPE_SD, "SD"),
    (BUS_TYPE_MMC, "MMC"),
    (BUS_TYPE_NVME, "NVMe"),
];

/// Whether a bus type is on the observed local-bus list.
#[cfg(any(windows, test))]
fn is_observed_bus(bus_type: u32) -> bool {
    OBSERVED_BUS_TYPES
        .iter()
        .any(|(observed, _)| *observed == bus_type)
}

/// One-line summary of a preliminary observation for the evidence bundle:
/// the drive type and the decoded device mapping. Used for the
/// before/after observation pair so the equality verdict is auditable.
#[cfg(any(windows, test))]
fn preliminary_summary(observation: &PreliminaryObservation) -> String {
    let mapping = match &observation.mapping {
        QueryState::Complete(units) => String::from_utf16_lossy(units),
        QueryState::ApiFailure => "api_failure".to_string(),
        QueryState::Partial => "partial".to_string(),
    };
    format!("drive_type {:?}, mapping {mapping}", observation.drive_type)
}

/// Observed-fact lines for every observed-but-unproven path (the demoted
/// grant-first path and the cause-specific matrix): the dependency-walk
/// observation, each disk extent, each required disk's bus type (numeric
/// value plus name) and removable-media flag with the query that produced
/// them, the before/after observation and equality verdict, the cleanup
/// outcome, the configured observed-bus list, the one-line P1–P4 mapping,
/// and the classification with its provenance. Recorded in a deterministic
/// order. A failed CloseHandle is cleanup failure, not evidence failure:
/// it is recorded as its own named fact and never wipes the observations.
/// Facts are observations only — they never admit. `disks` is `None` when
/// the disk-descriptor query failed: the unavailable facts are recorded
/// as unavailable, never silently dropped.
#[cfg(any(windows, test))]
fn observed_facts(
    before: &PreliminaryObservation,
    after: &PreliminaryObservation,
    extents: &[ExtentObservation],
    disks: Option<&[DiskQueryRecord]>,
    required_disks: &[u32],
    reason: &'static str,
    cleanup_failed: bool,
) -> Vec<String> {
    let mut facts = Vec::with_capacity(4 + extents.len() + required_disks.len());
    facts.push("dependency_walk: no_dependencies".to_string());
    for extent in extents {
        facts.push(format!(
            "extent: disk {} starting_offset {} extent_length {}",
            extent.disk_number, extent.starting_offset, extent.extent_length
        ));
    }
    for disk_number in required_disks {
        let record = disks.and_then(|records| {
            records
                .iter()
                .find(|record| record.disk_number == *disk_number)
        });
        match record.map(|record| record.outcome) {
            Some(DiskQueryOutcome::Observed(disk)) => facts.push(format!(
                "disk {}: bus_type {} ({}) removable_media {} (query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive{})",
                disk.disk_number,
                disk.bus_type,
                bus_type_label(disk.bus_type),
                disk.removable,
                disk.disk_number
            )),
            // Failed vs undecodable vs not-attempted vs open-failed are
            // distinct observations (WO29 Item 2): the bundle must not
            // claim every descriptor query failed when earlier queries
            // succeeded, a device never opened, or later queries never
            // ran.
            Some(DiskQueryOutcome::OpenFailed) => facts.push(format!(
                "disk {disk_number}: device open failed (no handle to PhysicalDrive{disk_number}); descriptor query never ran; bus_type unavailable, removable_media unavailable"
            )),
            Some(DiskQueryOutcome::Failed) => facts.push(format!(
                "disk {disk_number}: descriptor query failed (STORAGE_DEVICE_DESCRIPTOR.RemovableMedia via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive{disk_number}); bus_type unavailable, removable_media unavailable"
            )),
            Some(DiskQueryOutcome::Undecodable) => facts.push(format!(
                "disk {disk_number}: descriptor query undecodable (returned data failed validation via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive{disk_number}); bus_type unavailable, removable_media unavailable"
            )),
            Some(DiskQueryOutcome::NotAttempted) => facts.push(format!(
                "disk {disk_number}: descriptor query not attempted (an earlier disk query aborted the walk); bus_type unavailable, removable_media unavailable"
            )),
            None => facts.push(format!(
                "disk {}: bus_type unavailable, removable_media unavailable (query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia unreadable via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive{})",
                disk_number, disk_number
            )),
        }
    }
    let equality = if before == after { "match" } else { "mismatch" };
    facts.push(format!(
        "before: {}; after: {}; equality: {}",
        preliminary_summary(before),
        preliminary_summary(after),
        equality
    ));
    // A failed CloseHandle is cleanup failure, not evidence failure: the
    // observations above were already read, so they are preserved and
    // the cleanup failure is recorded as its own named fact rather than
    // wiping the observations.
    facts.push(if cleanup_failed {
        "cleanup: close_failed (CloseHandle reported failure after observations completed; observations preserved)"
            .to_string()
    } else {
        "cleanup: handles_closed".to_string()
    });
    facts.push(format!(
        "observed_bus_list: {}",
        OBSERVED_BUS_TYPES
            .iter()
            .map(|(value, name)| format!("{name}({value})"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    // P4 per decision 0029 §15: file-object ordinariness is enforced at
    // read time on the opened handle (`open_checked_windows_file`,
    // `run.rs` Step 4), never by this classifier — the classifier maps
    // P1 only. P2/P3/P4 outcomes are reported separately at read time.
    facts.push(format!(
        "P1-P4: P1 unproven ({reason}); P2-P4 enforced at read time, not by this classifier"
    ));
    facts.push(
        "classification: Unproven (observed-but-unproven; grant-first per decision 0029 §14; admits nothing)"
            .to_string(),
    );
    facts
}

/// Short label for a bus type, for the observed-fact record. The
/// observed-bus list (ATA/SATA/SD/MMC/NVMe) reports by name; anything
/// else is reported by number.
#[cfg(any(windows, test))]
fn bus_type_label(bus_type: u32) -> String {
    match bus_type {
        BUS_TYPE_ATA => "ATA".to_string(),
        BUS_TYPE_SATA => "SATA".to_string(),
        BUS_TYPE_SD => "SD".to_string(),
        BUS_TYPE_MMC => "MMC".to_string(),
        BUS_TYPE_NVME => "NVMe".to_string(),
        other => format!("bus_{other}"),
    }
}

#[cfg(any(windows, test))]
fn classify_evidence_detail(evidence: &InspectionEvidence) -> EvidenceVerdict {
    let PreliminaryClass::Candidate = classify_preliminary(&evidence.before) else {
        let PreliminaryClass::Closed(result) = classify_preliminary(&evidence.before) else {
            unreachable!();
        };
        return EvidenceVerdict::closed(result);
    };

    // A known before/after contradiction is contradictory evidence,
    // never missing evidence: the verdict stays closed (nothing
    // admits), but the contradiction marker and the before/after facts
    // are preserved so the runtime external-trust binder can refuse
    // them before payload consumption. Matching trust must never
    // reinterpret a known contradiction as acceptable absence.
    if evidence.before != evidence.after
        || classify_preliminary(&evidence.after) != PreliminaryClass::Candidate
    {
        return EvidenceVerdict::closed_contradiction(&evidence.before, &evidence.after);
    }

    if evidence.dependency != QueryState::Complete(DependencyObservation::None) {
        return EvidenceVerdict::closed(DriveLocality::Unknown);
    }

    let QueryState::Complete(extents) = &evidence.extents else {
        return EvidenceVerdict::closed(DriveLocality::Unknown);
    };
    if extents.is_empty()
        || extents
            .iter()
            .any(|extent| extent.starting_offset < 0 || extent.extent_length <= 0)
    {
        return EvidenceVerdict::closed(DriveLocality::Unknown);
    }

    let mut required_disks = extents
        .iter()
        .map(|extent| extent.disk_number)
        .collect::<Vec<_>>();
    required_disks.sort_unstable();
    required_disks.dedup();

    // The removable-status determination is unavailable, but the extent
    // topology was completely observed (WO29 Item 2): the cause is named
    // `Unproven`, with the unavailable facts recorded as unavailable —
    // never a reasonless `Unknown`.
    let QueryState::Complete(records) = &evidence.disks else {
        let facts = observed_facts(
            &evidence.before,
            &evidence.after,
            extents,
            None,
            &required_disks,
            REASON_REMOVABLE_STATUS_UNREADABLE,
            evidence.closes != QueryState::Complete(()),
        );
        return EvidenceVerdict::observed_unproven(
            REASON_REMOVABLE_STATUS_UNREADABLE,
            required_disks,
            facts,
        );
    };
    // Complete disk-record membership is validated BEFORE any cause
    // is selected (WO29 Item 2): the records must cover exactly the
    // required disk set — no missing disk, no extraneous disk, no
    // duplicate. A partial or contradictory record set is never a
    // complete observed matrix, so it cannot reach the cause-specific
    // `Unproven` paths; it fails closed. `required_disks` is
    // sorted/deduplicated, so sorted-record comparison is exact set
    // equality.
    let mut record_numbers: Vec<u32> = records.iter().map(|record| record.disk_number).collect();
    record_numbers.sort_unstable();
    if record_numbers != required_disks {
        return EvidenceVerdict::closed(DriveLocality::Unknown);
    }

    // Inner identity validation: an `Observed` record's inner
    // observation must name the same disk as the record's outer
    // `disk_number`. Required disk 0 with an outer record for disk 0
    // whose inner observation names disk 99 is not a complete observed
    // matrix — it fails closed before any cause is selected. (The live
    // producer always threads the walked disk number into the
    // descriptor decode, so a mismatch cannot arise honestly; the check
    // pins the invariant against reducer-side miswiring.)
    let inner_mismatch = records.iter().any(|record| {
        matches!(record.outcome, DiskQueryOutcome::Observed(disk) if disk.disk_number != record.disk_number)
    });
    if inner_mismatch {
        return EvidenceVerdict::closed(DriveLocality::Unknown);
    }

    // Per-disk outcome partition (WO29 Item 2): the determination is
    // only complete when every required disk's descriptor query was
    // observed. Failed, undecodable, not-attempted, and open-failed
    // queries keep their distinct observed facts — the available
    // observations are preserved, and the unavailable determination is
    // named `Unproven` with the unreadable reason, never a reasonless
    // `Unknown`.
    let complete = records
        .iter()
        .all(|record| matches!(record.outcome, DiskQueryOutcome::Observed(_)));
    if !complete {
        let facts = observed_facts(
            &evidence.before,
            &evidence.after,
            extents,
            Some(records),
            &required_disks,
            REASON_REMOVABLE_STATUS_UNREADABLE,
            evidence.closes != QueryState::Complete(()),
        );
        return EvidenceVerdict::observed_unproven(
            REASON_REMOVABLE_STATUS_UNREADABLE,
            required_disks,
            facts,
        );
    }

    // Cause-specific observed-but-unproven matrix (WO29 Item 2): every
    // required disk was observed, so the verdict names the cause instead
    // of a reasonless `Unknown`. First cause wins in sorted-disk order;
    // the observed facts still record every disk. Facts are observations
    // only — nothing here admits.
    let mut cause: Option<&'static str> = None;
    for disk_number in &required_disks {
        let Some(record) = records
            .iter()
            .find(|record| record.disk_number == *disk_number)
        else {
            // Unreachable: membership was validated above. Kept so the
            // loop stays total against the record shape.
            return EvidenceVerdict::closed(DriveLocality::Unknown);
        };
        let DiskQueryOutcome::Observed(disk) = record.outcome else {
            // Unreachable: every record was observed above. Kept so the
            // loop stays total against the outcome shape.
            return EvidenceVerdict::closed(DriveLocality::Unknown);
        };
        if disk.removable {
            cause = Some(REASON_REMOVABLE_MEDIA);
            break;
        }
        if !is_observed_bus(disk.bus_type) {
            cause = Some(REASON_UNSUPPORTED_BUS);
            break;
        }
    }

    // ------------------------------------------------------------------
    // GRANT-FIRST DEMOTION (decision 0029 §14, WO29 Slice A).
    //
    // Predecessor behavior: when every backing disk was observed as
    // non-removable with a bus type on the observed list (ATA/SATA/NVMe,
    // widened by Slice B to SD/MMC) over a completely-observed
    // extent/disk topology with no dependencies and observed closes, the
    // classifier emitted `(DriveLocality::FixedLocal, required_disks)` — a
    // positive admission that the drive was trusted-local.
    //
    // Why demoted: guest-visible bus-type observations do not establish
    // invisible backing. A hypervisor or other invisible intermediary can
    // interpose network/file backing beneath guest-visible observed-bus
    // frontends, and the guest cannot observe the difference. For this
    // WO29 version no classifier emits a positive admission: the
    // observation logic above is unchanged, only the verdict is demoted
    // to grant-first `Unproven` with the insufficient-evidence reason.
    // The observed disk numbers are kept as `backing_device_identity`
    // (device observation, not admission). `DriveLocality::FixedLocal`
    // stays defined (decision 0015's `proved` vocabulary) but is
    // unreachable from the live classifier — mirroring Linux's `Proven`.
    // ------------------------------------------------------------------
    let reason = cause.unwrap_or(REASON_INSUFFICIENT_EVIDENCE);
    let facts = observed_facts(
        &evidence.before,
        &evidence.after,
        extents,
        Some(records),
        &required_disks,
        reason,
        evidence.closes != QueryState::Complete(()),
    );
    EvidenceVerdict::observed_unproven(reason, required_disks, facts)
}

#[cfg(not(windows))]
pub fn classify(_root: DriveRoot) -> ClassifiedDrive {
    ClassifiedDrive {
        locality: DriveLocality::Unsupported,
        backing_device_identity: Vec::new(),
        observed_facts: Vec::new(),
        unproven_reason: None,
        volume_serial: None,
        contradiction: false,
    }
}

#[cfg(windows)]
pub fn classify(root: DriveRoot) -> ClassifiedDrive {
    let Some(volume) = open_device(&root.volume_device()) else {
        // The device cannot be opened: the volume serial is unobservable
        // and the full inspection below would report the same `Unknown`.
        return ClassifiedDrive {
            locality: DriveLocality::Unknown,
            backing_device_identity: Vec::new(),
            observed_facts: Vec::new(),
            unproven_reason: None,
            volume_serial: None,
            contradiction: false,
        };
    };
    // The serial is read from the SAME opened volume device the full
    // inspection below queries: one open, one coherent observation. Both
    // are the full 64-bit `FILE_ID_INFO` serial — never truncated, never
    // zero-extended.
    let volume_serial = query_volume_serial(volume.raw);
    let verdict = classify_full(root, volume);
    ClassifiedDrive {
        locality: verdict.locality,
        backing_device_identity: verdict.backing_device_identity,
        observed_facts: verdict.observed_facts,
        unproven_reason: verdict.unproven_reason,
        volume_serial,
        contradiction: verdict.contradiction,
    }
}

/// Full 64-bit volume serial from `FILE_ID_INFO` on an opened volume
/// device handle — the same unit as the file identity serial, so the
/// consumer binds classification and open with direct equality. Any
/// failure yields `None`: fail closed.
#[cfg(windows)]
fn query_volume_serial(volume_raw: *mut core::ffi::c_void) -> Option<u64> {
    file_id_info(volume_raw).map(|info| info.volume_serial_number)
}

/// Read one `FILE_ID_INFO` from a live handle via
/// `GetFileInformationByHandleEx` (`FileIdInfo`, class 18). Shared by the
/// file-identity reader and the volume-serial reader so both observe the
/// same unit. Returns `None` on null handle or API failure; the caller
/// applies its own validity rules (e.g. the zero-file-ID rejection).
#[cfg(windows)]
fn file_id_info(raw_handle: *mut core::ffi::c_void) -> Option<FileIdInfoLayout> {
    /// `FILE_INFO_BY_HANDLE_CLASS::FileIdInfo`.
    const FILE_ID_INFO_CLASS: u32 = 18;
    if raw_handle.is_null() {
        return None;
    }
    let mut info = core::mem::MaybeUninit::<FileIdInfoLayout>::uninit();
    let succeeded = unsafe {
        // SAFETY: `raw_handle` is non-null and, per the caller's contract,
        // a live open handle. `info` is a properly aligned 24-byte
        // out-buffer that stays alive for the call and is only read when
        // the call reports success.
        GetFileInformationByHandleEx(
            raw_handle,
            FILE_ID_INFO_CLASS,
            info.as_mut_ptr().cast(),
            core::mem::size_of::<FileIdInfoLayout>() as u32,
        )
    };
    if succeeded == 0 {
        return None;
    }
    Some(unsafe {
        // SAFETY: the API reported success, so the out-buffer is fully
        // initialized with one `FILE_ID_INFO`.
        info.assume_init()
    })
}

/// Identity of an already-opened file: the 64-bit volume serial plus the
/// 128-bit file reference number from `FILE_ID_INFO`
/// (`GetFileInformationByHandleEx` with `FileIdInfo`, class 18). This is
/// the documented stable identity contract: unlike the legacy 64-bit
/// `nFileIndex` from `GetFileInformationByHandle`, the 128-bit file ID is
/// unique and stable on ReFS, which is the filesystem this contract is
/// documented for.
///
/// `None` means the identity is unavailable (null handle, invalid handle,
/// API failure, a zero file ID, or a filesystem that does not expose
/// 128-bit file IDs) — never a sentinel value and never a downgrade to
/// the legacy index. There is deliberately no FAT/exFAT special case:
/// where the supported identity is unavailable or insufficient, the bind
/// fails closed.
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsFileIdentity {
    pub volume_serial: u64,
    pub file_id: [u8; 16],
}

/// Read the identity of an already-opened file handle via
/// `GetFileInformationByHandleEx` (`FileIdInfo`): `volume_serial` is
/// `FILE_ID_INFO.VolumeSerialNumber` and `file_id` is the 128-bit
/// `FILE_ID_INFO.FileId`. Returns `None` on any failure. Stable Rust
/// only; no new dependencies.
#[cfg(windows)]
pub fn opened_file_identity(raw_handle: *mut core::ffi::c_void) -> Option<WindowsFileIdentity> {
    let info = file_id_info(raw_handle)?;
    if info.file_id == [0; 16] {
        // A zero file ID is not a usable identity: the filesystem did
        // not provide one. Fail closed rather than bind against a
        // sentinel.
        return None;
    }
    Some(WindowsFileIdentity {
        volume_serial: info.volume_serial_number,
        file_id: info.file_id,
    })
}

/// Read the identity of the file at `path` (a NUL-terminated UTF-16 path)
/// WITHOUT following reparse points: the handle is opened with
/// `FILE_FLAG_OPEN_REPARSE_POINT`, so a symlink or mount-point reparse at
/// the final component yields the reparse point's own identity, not its
/// target's. Returns `None` on any failure. Stable Rust only; the FFI is
/// confined to this crate's audited `kernel32` block.
#[cfg(windows)]
pub fn walked_file_identity(path_nul_terminated_utf16: &[u16]) -> Option<WindowsFileIdentity> {
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    if path_nul_terminated_utf16.last() != Some(&0) {
        return None;
    }
    let handle = unsafe {
        // SAFETY: the path is NUL-terminated per the checked precondition
        // and remains alive for the call. Desired access is zero (metadata
        // only); share mode allows concurrent readers/writers/deleters;
        // `OPEN_EXISTING` never creates; the reparse-point flag prevents
        // link following; backup semantics permits directory handles.
        CreateFileW(
            path_nul_terminated_utf16.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS,
            core::ptr::null_mut(),
        )
    };
    if handle.is_null() || handle == (-1isize as *mut core::ffi::c_void) {
        return None;
    }
    let identity = opened_file_identity(handle);
    unsafe {
        // SAFETY: `handle` came from the successful `CreateFileW` above and
        // is closed exactly once here.
        CloseHandle(handle);
    }
    identity
}

/// One coherent observation of the volume actually containing an opened
/// file: the full 64-bit volume serial from the opened file handle's own
/// `FILE_ID_INFO` plus the complete sorted/deduplicated extent set from
/// the volume device resolved from the handle's volume-GUID final path.
/// The two fields are observed together so the consumer binds the serial
/// and the extents as a single unit; neither is ever re-derived from the
/// walk's drive letter (which names the wrong volume for folder-mounted
/// and redirected topology).
#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedVolumeObservation {
    /// Full 64-bit serial from `FILE_ID_INFO` on the opened file handle —
    /// the same unit as the opened file identity's serial.
    pub volume_serial: u64,
    /// Sorted, deduplicated physical disk numbers backing the volume.
    pub disk_numbers: Vec<u32>,
}

/// Resolve the volume actually containing an already-opened file, from
/// the OPENED OBJECT — never re-derived from the classified path. The
/// serial is read on the opened file handle's own `FILE_ID_INFO` — the
/// same unit as the opened file identity, queried on the same handle —
/// never on the volume-device handle (that query is unreliable natively).
/// The handle's volume-GUID final path names the true containing volume
/// (correct for folder-mounted and redirected topology, where reopening
/// a drive letter can name the wrong volume); the volume device is
/// opened by GUID and the extent list is read live from that device,
/// with the same decoder the classifier uses.
///
/// `None` means the observation is unavailable (null handle, API failure,
/// a final path that is not a volume-GUID form, or an empty extent set)
/// — never a partial or invented list.
#[cfg(windows)]
pub fn opened_volume_observation(
    raw_handle: *mut core::ffi::c_void,
) -> Option<OpenedVolumeObservation> {
    if raw_handle.is_null() {
        return None;
    }
    // The serial comes from the OPENED FILE handle's own `FILE_ID_INFO`:
    // the same handle that produced the file identity, the same unit the
    // consumer binds with direct equality — no truncation, no
    // zero-extension, and no unreliable volume-device query.
    let serial = file_id_info(raw_handle)?.volume_serial_number;
    // The containing volume device, derived from the handle's own
    // volume-GUID final path — never re-derived from a drive letter.
    let guid_path = volume_guid_path_by_handle(raw_handle)?;
    let device = containing_volume_device_path(&guid_path)?;
    let volume = open_device(&device)?;
    let mut buffer = Box::new(AlignedBuffer::<EXTENT_BUFFER_BYTES>(
        [0; EXTENT_BUFFER_BYTES],
    ));
    let returned = device_io_control(
        &volume,
        IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
        &[],
        &mut buffer.0,
    );
    let QueryState::Complete(extents) = decode_extent_information(&buffer.0, returned) else {
        return None;
    };
    let mut disk_numbers: Vec<u32> = extents.iter().map(|extent| extent.disk_number).collect();
    disk_numbers.sort_unstable();
    disk_numbers.dedup();
    if disk_numbers.is_empty() {
        let _already_unknown = volume.close();
        return None;
    }
    // Explicit close per the crate's handle convention: the observation is
    // complete (serial + extents already read), so the close result is
    // cleanup, not evidence — named as already-unknown rather than left to
    // `Drop`.
    let _already_unknown = volume.close();
    Some(OpenedVolumeObservation {
        volume_serial: serial,
        disk_numbers,
    })
}

/// The handle's final path in volume-GUID form, without the trailing
/// NUL. Two-call pattern: the first call reports the required length,
/// the second fills the buffer. `None` on any failure.
#[cfg(windows)]
fn volume_guid_path_by_handle(raw_handle: *mut core::ffi::c_void) -> Option<Vec<u16>> {
    /// `VOLUME_NAME_GUID`.
    const GUID_VOLUME_NAME: u32 = 0x1;
    /// Sanity bound: a final path longer than this is not a usable
    /// observation.
    const MAX_FINAL_PATH_UNITS: u32 = 32 * 1024;
    let needed = unsafe {
        // SAFETY: `raw_handle` is a live open handle per the caller's
        // contract; a null buffer with zero capacity only queries the
        // required length and writes nothing.
        GetFinalPathNameByHandleW(raw_handle, core::ptr::null_mut(), 0, GUID_VOLUME_NAME)
    };
    if needed == 0 || needed > MAX_FINAL_PATH_UNITS {
        return None;
    }
    let mut buffer = vec![0u16; needed as usize];
    let written = unsafe {
        // SAFETY: `buffer` offers exactly `needed` writable units and
        // stays alive for the call; `raw_handle` is live.
        GetFinalPathNameByHandleW(raw_handle, buffer.as_mut_ptr(), needed, GUID_VOLUME_NAME)
    };
    if written == 0 || written >= needed {
        return None;
    }
    buffer.truncate(written as usize);
    Some(buffer)
}

/// Build the NUL-terminated Win32 device-namespace path for the volume
/// containing the file named by a volume-GUID final path. The real
/// `GetFinalPathNameByHandleW` `VOLUME_NAME_GUID` form names the full
/// file path inside the volume (volume GUID followed by the in-volume
/// path), not a volume root: the containing volume's device path is the
/// GUID root through the closing brace, plus the NUL; the file's path
/// within the volume is dropped. The root-only form is accepted too. The
/// path is validated structurally (GUID prefix, non-empty brace-closed
/// GUID body); the exact GUID text is the OS's to validate when the
/// device is opened. Anything else is `None`.
#[cfg(any(windows, test))]
fn containing_volume_device_path(guid_path: &[u16]) -> Option<Vec<u16>> {
    const PREFIX: [u16; 11] = [
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'V' as u16,
        b'o' as u16,
        b'l' as u16,
        b'u' as u16,
        b'm' as u16,
        b'e' as u16,
        b'{' as u16,
    ];
    if guid_path.len() < PREFIX.len() + 1 || guid_path[..PREFIX.len()] != PREFIX {
        return None;
    }
    // The GUID body runs from the prefix through the first closing brace.
    // Everything after that brace is the file's path within the volume
    // and is dropped; the body must be non-empty.
    let body_and_rest = &guid_path[PREFIX.len()..];
    let brace_offset = body_and_rest
        .iter()
        .position(|&unit| unit == u16::from(b'}'))?;
    if brace_offset == 0 {
        return None;
    }
    let root_end = PREFIX.len() + brace_offset + 1;
    let mut device: Vec<u16> = guid_path[..root_end].to_vec();
    device.push(0);
    Some(device)
}

#[cfg(windows)]
fn classify_full(root: DriveRoot, volume: OwnedHandle) -> EvidenceVerdict {
    let before = query_preliminary(root);
    if let PreliminaryClass::Closed(result) = classify_preliminary(&before) {
        let _already_unknown = volume.close();
        return EvidenceVerdict::closed(result);
    }

    let dependency = query_dependencies(&volume);
    if dependency != QueryState::Complete(DependencyObservation::None) {
        let _already_unknown = volume.close();
        return EvidenceVerdict::closed(DriveLocality::Unknown);
    }
    let extents = query_extents(&volume);
    let (disks, disk_closes) = match &extents {
        QueryState::Complete(extents) => query_backing_disks(extents),
        QueryState::ApiFailure => (QueryState::ApiFailure, true),
        QueryState::Partial => (QueryState::Partial, true),
    };
    let volume_closed = volume.close();
    let after = query_preliminary(root);

    classify_evidence_detail(&InspectionEvidence {
        before,
        dependency,
        extents,
        disks,
        closes: if volume_closed && disk_closes {
            QueryState::Complete(())
        } else {
            QueryState::ApiFailure
        },
        after,
    })
}

#[cfg(any(windows, test))]
fn is_ordinary_fixed_target(target: &[u16]) -> bool {
    const PREFIX: &str = r"\Device\HarddiskVolume";
    let Some(suffix) = strip_ascii_prefix_case_insensitive(target, PREFIX) else {
        return false;
    };
    !suffix.is_empty()
        && suffix
            .iter()
            .all(|unit| *unit >= u16::from(b'0') && *unit <= u16::from(b'9'))
}

#[cfg(any(windows, test))]
fn starts_ascii_case_insensitive(value: &[u16], prefix: &str) -> bool {
    strip_ascii_prefix_case_insensitive(value, prefix).is_some()
}

#[cfg(any(windows, test))]
fn strip_ascii_prefix_case_insensitive<'a>(value: &'a [u16], prefix: &str) -> Option<&'a [u16]> {
    let bytes = prefix.as_bytes();
    if value.len() < bytes.len() {
        return None;
    }
    value
        .iter()
        .zip(bytes)
        .take(bytes.len())
        .all(|(actual, expected)| {
            *actual <= u16::from(u8::MAX) && (*actual as u8).eq_ignore_ascii_case(expected)
        })
        .then_some(&value[bytes.len()..])
}

#[cfg(test)]
const BUS_TYPE_SCSI: u32 = 1;
#[cfg(any(windows, test))]
const BUS_TYPE_ATA: u32 = 3;
#[cfg(test)]
const BUS_TYPE_FIBRE: u32 = 6;
#[cfg(test)]
const BUS_TYPE_RAID: u32 = 8;
#[cfg(test)]
const BUS_TYPE_ISCSI: u32 = 9;
#[cfg(test)]
const BUS_TYPE_SAS: u32 = 10;
#[cfg(any(windows, test))]
const BUS_TYPE_SATA: u32 = 11;
// WO29 Slice B (Item 2): SD (12) and MMC (13) join the observed bus
// list. Observation only — no bus type independently earns `proved`.
#[cfg(any(windows, test))]
const BUS_TYPE_SD: u32 = 12;
#[cfg(any(windows, test))]
const BUS_TYPE_MMC: u32 = 13;
#[cfg(test)]
const BUS_TYPE_VIRTUAL: u32 = 14;
#[cfg(test)]
const BUS_TYPE_FILE_BACKED_VIRTUAL: u32 = 15;
#[cfg(test)]
const BUS_TYPE_SPACES: u32 = 16;
#[cfg(any(windows, test))]
const BUS_TYPE_NVME: u32 = 17;
#[cfg(test)]
const BUS_TYPE_NVMEOF: u32 = 20;

#[cfg(windows)]
const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x0056_0000;
#[cfg(windows)]
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002d_1400;

#[cfg(windows)]
const QUERY_BUFFER_UNITS: usize = 1024;
#[cfg(windows)]
const DEPENDENCY_BUFFER_BYTES: usize = 64 * 1024;
#[cfg(windows)]
const EXTENT_BUFFER_BYTES: usize = 64 * 1024;
#[cfg(windows)]
const DESCRIPTOR_BUFFER_BYTES: usize = 4096;
#[cfg(windows)]
const MAX_EXTENTS: usize = 1024;

#[cfg(windows)]
#[repr(C, align(8))]
struct AlignedBuffer<const N: usize>([u8; N]);

#[cfg(windows)]
struct OwnedHandle {
    raw: *mut core::ffi::c_void,
    closed: bool,
}

#[cfg(windows)]
impl OwnedHandle {
    fn close(mut self) -> bool {
        let closed = close_raw_handle(self.raw);
        self.closed = true;
        closed
    }
}

#[cfg(windows)]
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.closed {
            // This is a best-effort unwind/early-failure fallback. Every path
            // that can reach the demoted grant-first admission verdict calls
            // `close`, observes its result, and marks the handle closed
            // before Drop.
            let _already_unknown = close_raw_handle(self.raw);
            self.closed = true;
        }
    }
}

#[cfg(windows)]
fn close_raw_handle(raw: *mut core::ffi::c_void) -> bool {
    unsafe {
        // SAFETY: raw came from one successful CreateFileW call. This helper is
        // reached exactly once for each OwnedHandle, either by explicit
        // observed close or by the already-failing Drop fallback.
        CloseHandle(raw) != 0
    }
}

#[cfg(windows)]
fn query_preliminary(root: DriveRoot) -> PreliminaryObservation {
    let drive_type = unsafe {
        // SAFETY: DriveRoot owns a four-unit NUL-terminated drive root for
        // the duration of the read-only GetDriveTypeW call.
        GetDriveTypeW(root.root.as_ptr())
    };
    let mut buffer = [0u16; QUERY_BUFFER_UNITS];
    let returned = unsafe {
        // SAFETY: DriveRoot owns a NUL-terminated drive device name. The
        // writable output and the reported capacity are exactly equal.
        QueryDosDeviceW(
            root.device.as_ptr(),
            buffer.as_mut_ptr(),
            QUERY_BUFFER_UNITS as u32,
        )
    };
    PreliminaryObservation {
        drive_type: drive_type_observation(drive_type),
        mapping: decode_device_mapping(&buffer, returned),
    }
}

#[cfg(windows)]
fn open_device(name: &[u16]) -> Option<OwnedHandle> {
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
    let handle = unsafe {
        // SAFETY: name is synthesized by this crate, NUL-terminated, and
        // remains alive for the call. Desired access is zero and the template
        // and security pointers are null.
        CreateFileW(
            name.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        )
    };
    (!handle.is_null() && handle != (-1isize as *mut core::ffi::c_void)).then_some(OwnedHandle {
        raw: handle,
        closed: false,
    })
}

#[cfg(windows)]
fn device_io_control(
    handle: &OwnedHandle,
    control_code: u32,
    input: &[u8],
    output: &mut [u8],
) -> Option<usize> {
    let mut returned = 0u32;
    let input_pointer = if input.is_empty() {
        core::ptr::null_mut()
    } else {
        input.as_ptr().cast_mut().cast()
    };
    let success = unsafe {
        // SAFETY: handle is live; the input and output slices remain valid for
        // their exact lengths; returned is writable; no OVERLAPPED is used.
        DeviceIoControl(
            handle.raw,
            control_code,
            input_pointer,
            input.len() as u32,
            output.as_mut_ptr().cast(),
            output.len() as u32,
            &mut returned,
            core::ptr::null_mut(),
        )
    };
    (success != 0 && (returned as usize) <= output.len()).then_some(returned as usize)
}

#[cfg(windows)]
fn query_dependencies(handle: &OwnedHandle) -> QueryState<DependencyObservation> {
    const STORAGE_DEPENDENCY_INFO_VERSION_2: u32 = 2;
    const GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES: u32 = 1;
    let mut buffer = Box::new(AlignedBuffer::<DEPENDENCY_BUFFER_BYTES>(
        [0; DEPENDENCY_BUFFER_BYTES],
    ));
    buffer.0[..4].copy_from_slice(&STORAGE_DEPENDENCY_INFO_VERSION_2.to_ne_bytes());
    let mut size_used = 0u32;
    let status = unsafe {
        // SAFETY: handle is a live volume handle. The aligned buffer is
        // writable for the exact bounded size supplied, begins with the type-2
        // version tag, and size_used remains valid for the call.
        GetStorageDependencyInformation(
            handle.raw,
            GET_STORAGE_DEPENDENCY_FLAG_HOST_VOLUMES,
            DEPENDENCY_BUFFER_BYTES as u32,
            buffer.0.as_mut_ptr().cast(),
            &mut size_used,
        )
    };
    decode_dependency_information(&buffer.0, status, size_used)
}

#[cfg(windows)]
fn decode_dependency_information(
    buffer: &[u8],
    status: u32,
    size_used: u32,
) -> QueryState<DependencyObservation> {
    const STORAGE_DEPENDENCY_INFO_VERSION_2: u32 = 2;
    const HEADER_BYTES: usize = 8;
    const ENTRY_BYTES: usize = core::mem::size_of::<StorageDependencyInfoType2Layout>();
    const MAX_DEPENDENCIES: usize = 1024;
    if status != 0 {
        return QueryState::ApiFailure;
    }
    let size_used = size_used as usize;
    if !(HEADER_BYTES..=buffer.len()).contains(&size_used)
        || read_u32(buffer, 0) != Some(STORAGE_DEPENDENCY_INFO_VERSION_2)
    {
        return QueryState::Partial;
    }
    let Some(count) = read_u32(buffer, 4).map(|value| value as usize) else {
        return QueryState::Partial;
    };
    if count > MAX_DEPENDENCIES {
        return QueryState::Partial;
    }
    let Some(required) = count
        .checked_mul(ENTRY_BYTES)
        .and_then(|bytes| HEADER_BYTES.checked_add(bytes))
    else {
        return QueryState::Partial;
    };
    if size_used < required {
        return QueryState::Partial;
    }
    if count == 0 {
        QueryState::Complete(DependencyObservation::None)
    } else {
        QueryState::Complete(DependencyObservation::Present)
    }
}

#[cfg(windows)]
fn query_extents(handle: &OwnedHandle) -> QueryState<Vec<ExtentObservation>> {
    let mut buffer = Box::new(AlignedBuffer::<EXTENT_BUFFER_BYTES>(
        [0; EXTENT_BUFFER_BYTES],
    ));
    let returned = device_io_control(
        handle,
        IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
        &[],
        &mut buffer.0,
    );
    decode_extent_information(&buffer.0, returned)
}

#[cfg(windows)]
fn decode_extent_information(
    buffer: &[u8],
    returned: Option<usize>,
) -> QueryState<Vec<ExtentObservation>> {
    const EXTENTS_OFFSET: usize = core::mem::offset_of!(VolumeDiskExtentsLayout, extent);
    const EXTENT_SIZE: usize = core::mem::size_of::<DiskExtentLayout>();
    const START_OFFSET: usize = core::mem::offset_of!(DiskExtentLayout, starting_offset);
    const LENGTH_OFFSET: usize = core::mem::offset_of!(DiskExtentLayout, extent_length);
    let Some(returned) = returned else {
        return QueryState::ApiFailure;
    };
    if returned > buffer.len() {
        return QueryState::Partial;
    }
    let view = &buffer[..returned];
    let Some(count) = read_u32(view, 0).map(|value| value as usize) else {
        return QueryState::Partial;
    };
    if count == 0 || count > MAX_EXTENTS {
        return QueryState::Partial;
    }
    let Some(required) = count
        .checked_mul(EXTENT_SIZE)
        .and_then(|bytes| EXTENTS_OFFSET.checked_add(bytes))
    else {
        return QueryState::Partial;
    };
    if returned < required {
        return QueryState::Partial;
    }
    let mut extents = Vec::with_capacity(count);
    for index in 0..count {
        let base = EXTENTS_OFFSET + index * EXTENT_SIZE;
        let (Some(disk_number), Some(starting_offset), Some(extent_length)) = (
            read_u32(view, base),
            read_i64(view, base + START_OFFSET),
            read_i64(view, base + LENGTH_OFFSET),
        ) else {
            return QueryState::Partial;
        };
        extents.push(ExtentObservation {
            disk_number,
            starting_offset,
            extent_length,
        });
    }
    QueryState::Complete(extents)
}

#[cfg(windows)]
fn query_backing_disks(extents: &[ExtentObservation]) -> (QueryState<Vec<DiskQueryRecord>>, bool) {
    let mut numbers = extents
        .iter()
        .map(|extent| extent.disk_number)
        .collect::<Vec<_>>();
    numbers.sort_unstable();
    numbers.dedup();
    if numbers.is_empty() || numbers.len() > MAX_EXTENTS {
        return (QueryState::Partial, true);
    }

    // One record per required disk, in walk order. The walk aborts on
    // the first failed query — the failed disk keeps its `Failed` /
    // `Undecodable` outcome and every disk after it is recorded as
    // `NotAttempted`, so the reducer never claims a query failed that
    // never ran (WO29 Item 2).
    let mut records = Vec::with_capacity(numbers.len());
    let mut closes = true;
    for (index, disk_number) in numbers.iter().enumerate() {
        let mut name = vec![
            u16::from(b'\\'),
            u16::from(b'\\'),
            u16::from(b'.'),
            u16::from(b'\\'),
        ];
        name.extend("PhysicalDrive".encode_utf16());
        name.extend(disk_number.to_string().encode_utf16());
        name.push(0);
        let outcome = match open_device(&name) {
            // The device could not be opened: no descriptor query ran.
            // This is distinct from a failed query — the evidence must
            // never claim a descriptor query failed that never ran.
            None => DiskQueryOutcome::OpenFailed,
            Some(handle) => {
                let query = query_disk_descriptor(&handle, *disk_number);
                let closed = handle.close();
                if !closed {
                    closes = false;
                }
                // A failed CloseHandle is cleanup failure, not evidence
                // failure: a successful observation is preserved and the
                // walk continues, because the observations were already
                // read. The cleanup failure is carried separately as a
                // named fact — a successful observation must not
                // disappear merely because close failed.
                match query {
                    QueryState::Complete(disk) => DiskQueryOutcome::Observed(disk),
                    QueryState::ApiFailure => DiskQueryOutcome::Failed,
                    QueryState::Partial => DiskQueryOutcome::Undecodable,
                }
            }
        };
        records.push(DiskQueryRecord {
            disk_number: *disk_number,
            outcome,
        });
        if !matches!(outcome, DiskQueryOutcome::Observed(_)) {
            // Abort the walk: later disks were never queried.
            for remaining in &numbers[index + 1..] {
                records.push(DiskQueryRecord {
                    disk_number: *remaining,
                    outcome: DiskQueryOutcome::NotAttempted,
                });
            }
            return (QueryState::Complete(records), closes);
        }
    }
    (QueryState::Complete(records), closes)
}

#[cfg(windows)]
fn query_disk_descriptor(handle: &OwnedHandle, disk_number: u32) -> QueryState<DiskObservation> {
    let query = [0u8; 12];
    let mut buffer = Box::new(AlignedBuffer::<DESCRIPTOR_BUFFER_BYTES>(
        [0; DESCRIPTOR_BUFFER_BYTES],
    ));
    let returned = device_io_control(handle, IOCTL_STORAGE_QUERY_PROPERTY, &query, &mut buffer.0);
    decode_disk_descriptor(&buffer.0, returned, disk_number)
}

#[cfg(windows)]
fn decode_disk_descriptor(
    buffer: &[u8],
    returned: Option<usize>,
    disk_number: u32,
) -> QueryState<DiskObservation> {
    const BUS_OFFSET: usize = core::mem::offset_of!(StorageDeviceDescriptorLayout, bus_type);
    const REMOVABLE_OFFSET: usize =
        core::mem::offset_of!(StorageDeviceDescriptorLayout, removable_media);
    const MINIMUM: usize = core::mem::size_of::<StorageDeviceDescriptorLayout>();
    let Some(returned) = returned else {
        return QueryState::ApiFailure;
    };
    if returned > buffer.len() {
        return QueryState::Partial;
    }
    let view = &buffer[..returned];
    let (Some(version), Some(size), Some(bus_type)) = (
        read_u32(view, 0),
        read_u32(view, 4).map(|value| value as usize),
        read_u32(view, BUS_OFFSET),
    ) else {
        return QueryState::Partial;
    };
    if version < MINIMUM as u32 || size < MINIMUM || size > returned {
        return QueryState::Partial;
    }
    let Some(removable) = view.get(REMOVABLE_OFFSET) else {
        return QueryState::Partial;
    };
    QueryState::Complete(DiskObservation {
        disk_number,
        removable: *removable != 0,
        bus_type,
    })
}

#[cfg(windows)]
fn drive_type_observation(raw: u32) -> DriveTypeObservation {
    match raw {
        0 => DriveTypeObservation::Unknown,
        1 => DriveTypeObservation::MissingRoot,
        2 => DriveTypeObservation::Removable,
        3 => DriveTypeObservation::Fixed,
        4 => DriveTypeObservation::Remote,
        5 => DriveTypeObservation::Optical,
        6 => DriveTypeObservation::RamDisk,
        _ => DriveTypeObservation::Other,
    }
}

#[cfg(windows)]
fn decode_device_mapping(buffer: &[u16], returned: u32) -> QueryState<Vec<u16>> {
    let returned = returned as usize;
    if returned == 0 {
        return QueryState::ApiFailure;
    }
    if returned > buffer.len() {
        return QueryState::Partial;
    }
    let initialized = &buffer[..returned];
    let Some(first_nul) = initialized.iter().position(|unit| *unit == 0) else {
        return QueryState::Partial;
    };
    if first_nul == 0 || initialized[first_nul + 1..].iter().any(|unit| *unit != 0) {
        return QueryState::Partial;
    }
    QueryState::Complete(initialized[..first_nul].to_vec())
}

#[cfg(windows)]
fn read_u32(buffer: &[u8], offset: usize) -> Option<u32> {
    buffer
        .get(offset..offset.checked_add(4)?)?
        .try_into()
        .ok()
        .map(u32::from_ne_bytes)
}

#[cfg(windows)]
fn read_i64(buffer: &[u8], offset: usize) -> Option<i64> {
    buffer
        .get(offset..offset.checked_add(8)?)?
        .try_into()
        .ok()
        .map(i64::from_ne_bytes)
}

#[cfg(windows)]
#[repr(C)]
struct DiskExtentLayout {
    disk_number: u32,
    starting_offset: i64,
    extent_length: i64,
}

#[cfg(windows)]
#[repr(C)]
struct VolumeDiskExtentsLayout {
    count: u32,
    extent: DiskExtentLayout,
}

#[cfg(windows)]
#[repr(C)]
struct GuidLayout {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(windows)]
#[repr(C)]
struct VirtualStorageTypeLayout {
    device_id: u32,
    vendor_id: GuidLayout,
}

#[cfg(windows)]
#[repr(C)]
struct StorageDependencyInfoType2Layout {
    dependency_type_flags: u32,
    provider_specific_flags: u32,
    virtual_storage_type: VirtualStorageTypeLayout,
    ancestor_level: u32,
    dependency_device_name: *mut u16,
    host_volume_name: *mut u16,
    dependent_volume_name: *mut u16,
    dependent_volume_relative_path: *mut u16,
}

#[cfg(windows)]
#[repr(C)]
struct StorageDeviceDescriptorLayout {
    version: u32,
    size: u32,
    device_type: u8,
    device_type_modifier: u8,
    removable_media: u8,
    command_queueing: u8,
    vendor_id_offset: u32,
    product_id_offset: u32,
    product_revision_offset: u32,
    serial_number_offset: u32,
    bus_type: u32,
    raw_properties_length: u32,
}

/// `FILE_ID_INFO` as documented by the Windows API: the 64-bit volume
/// serial number followed by the 128-bit file reference number. `repr(C)`
/// gives the exact 24-byte Windows layout on every target.
#[cfg(windows)]
#[repr(C)]
struct FileIdInfoLayout {
    volume_serial_number: u64,
    file_id: [u8; 16],
}

#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn GetDriveTypeW(lp_root_path_name: *const u16) -> u32;
    fn QueryDosDeviceW(lp_device_name: *const u16, lp_target_path: *mut u16, ucch_max: u32) -> u32;
    fn CreateFileW(
        lp_file_name: *const u16,
        desired_access: u32,
        share_mode: u32,
        security_attributes: *mut core::ffi::c_void,
        creation_disposition: u32,
        flags_and_attributes: u32,
        template_file: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
    fn DeviceIoControl(
        device: *mut core::ffi::c_void,
        control_code: u32,
        input: *mut core::ffi::c_void,
        input_size: u32,
        output: *mut core::ffi::c_void,
        output_size: u32,
        bytes_returned: *mut u32,
        overlapped: *mut core::ffi::c_void,
    ) -> i32;
    fn CloseHandle(object: *mut core::ffi::c_void) -> i32;
    fn GetFileInformationByHandleEx(
        file: *mut core::ffi::c_void,
        file_information_class: u32,
        file_information: *mut core::ffi::c_void,
        buffer_size: u32,
    ) -> i32;
    fn GetFinalPathNameByHandleW(
        file: *mut core::ffi::c_void,
        file_path: *mut u16,
        file_path_size: u32,
        flags: u32,
    ) -> u32;
}

#[cfg(windows)]
#[link(name = "VirtDisk")]
unsafe extern "system" {
    fn GetStorageDependencyInformation(
        object: *mut core::ffi::c_void,
        flags: u32,
        storage_dependency_info_size: u32,
        storage_dependency_info: *mut core::ffi::c_void,
        size_used: *mut u32,
    ) -> u32;
}

#[cfg(test)]
mod tests {
    use super::{
        BUS_TYPE_ATA, BUS_TYPE_FIBRE, BUS_TYPE_FILE_BACKED_VIRTUAL, BUS_TYPE_ISCSI, BUS_TYPE_MMC,
        BUS_TYPE_NVME, BUS_TYPE_NVMEOF, BUS_TYPE_RAID, BUS_TYPE_SAS, BUS_TYPE_SATA, BUS_TYPE_SCSI,
        BUS_TYPE_SD, BUS_TYPE_SPACES, BUS_TYPE_VIRTUAL, DependencyObservation, DiskObservation,
        DiskQueryOutcome, DiskQueryRecord, DriveLocality, DriveRoot, DriveTypeObservation,
        ExtentObservation, InspectionEvidence, PreliminaryObservation, QueryState,
        REASON_INSUFFICIENT_EVIDENCE, REASON_REMOVABLE_MEDIA, REASON_REMOVABLE_STATUS_UNREADABLE,
        REASON_UNSUPPORTED_BUS, classify_evidence, classify_evidence_detail,
    };
    // `ClassifiedDrive` is only constructed by the non-Windows test below;
    // on the Windows target that test is cfg'd out.
    #[cfg(not(windows))]
    use super::ClassifiedDrive;

    fn mapping(text: &str) -> QueryState<Vec<u16>> {
        QueryState::Complete(text.encode_utf16().collect())
    }

    #[cfg(windows)]
    fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
        buffer[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
    }

    #[cfg(windows)]
    fn put_i64(buffer: &mut [u8], offset: usize, value: i64) {
        buffer[offset..offset + 8].copy_from_slice(&value.to_ne_bytes());
    }

    fn preliminary() -> PreliminaryObservation {
        PreliminaryObservation {
            drive_type: DriveTypeObservation::Fixed,
            mapping: mapping(r"\Device\HarddiskVolume3"),
        }
    }

    fn observed_record(disk_number: u32, removable: bool, bus_type: u32) -> DiskQueryRecord {
        DiskQueryRecord {
            disk_number,
            outcome: DiskQueryOutcome::Observed(DiskObservation {
                disk_number,
                removable,
                bus_type,
            }),
        }
    }

    /// Marks the first disk record as observed-removable (keeps the bus
    /// type); the record must already be an `Observed` outcome.
    fn make_removable(evidence: &mut InspectionEvidence) {
        let QueryState::Complete(records) = &mut evidence.disks else {
            unreachable!("removable fixtures need complete disk records");
        };
        let DiskQueryOutcome::Observed(disk) = &mut records[0].outcome else {
            unreachable!("removable fixtures need an observed first disk");
        };
        disk.removable = true;
    }

    fn evidence(bus_type: u32) -> InspectionEvidence {
        InspectionEvidence {
            before: preliminary(),
            dependency: QueryState::Complete(DependencyObservation::None),
            extents: QueryState::Complete(vec![ExtentObservation {
                disk_number: 0,
                starting_offset: 1_048_576,
                extent_length: 4_194_304,
            }]),
            disks: QueryState::Complete(vec![observed_record(0, false, bus_type)]),
            closes: QueryState::Complete(()),
            after: preliminary(),
        }
    }

    /// Multi-disk evidence: `required` are the disk numbers named by the
    /// extent topology; `records` are the descriptor-query records in
    /// walk order (they may disagree with `required` — that is what the
    /// membership validation must catch).
    fn multi_disk_evidence(required: &[u32], records: Vec<DiskQueryRecord>) -> InspectionEvidence {
        let mut evidence = evidence(BUS_TYPE_NVME);
        evidence.extents = QueryState::Complete(
            required
                .iter()
                .map(|disk_number| ExtentObservation {
                    disk_number: *disk_number,
                    starting_offset: 0,
                    extent_length: 4096,
                })
                .collect(),
        );
        evidence.disks = QueryState::Complete(records);
        evidence
    }

    #[test]
    fn drive_root_accepts_only_ascii_letters() {
        assert!(DriveRoot::from_ascii_letter(b'C').is_some());
        assert!(DriveRoot::from_ascii_letter(b'z').is_some());
        for malformed in [0, b'0', b':', b'\\', 0xff] {
            assert!(DriveRoot::from_ascii_letter(malformed).is_none());
        }
    }

    #[test]
    fn observed_bus_chains_are_unproven_with_observed_facts() {
        // WO29 Slice B widens the observed bus list to SD/MMC: observation
        // only — the verdict stays grant-first `Unproven`.
        for (bus, value, label) in [
            (BUS_TYPE_ATA, "3", "ATA"),
            (BUS_TYPE_SATA, "11", "SATA"),
            (BUS_TYPE_SD, "12", "SD"),
            (BUS_TYPE_MMC, "13", "MMC"),
            (BUS_TYPE_NVME, "17", "NVMe"),
        ] {
            let detail = classify_evidence_detail(&evidence(bus));
            assert_eq!(detail.locality, DriveLocality::Unproven, "{label}");
            assert_eq!(
                detail.unproven_reason,
                Some(REASON_INSUFFICIENT_EVIDENCE),
                "{label}"
            );
            // The demoted path keeps the observed disk numbers as
            // backing-device identity (observation, not admission).
            assert_eq!(detail.backing_device_identity, vec![0], "{label}");
            // The disk fact carries the numeric bus value plus the name.
            assert!(
                detail.observed_facts.iter().any(|fact| fact
                    == &format!("disk 0: bus_type {value} ({label}) removable_media false (query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive0)")),
                "{label}: {facts:?}",
                facts = detail.observed_facts
            );
        }
        for bus in [
            BUS_TYPE_ATA,
            BUS_TYPE_SATA,
            BUS_TYPE_SD,
            BUS_TYPE_MMC,
            BUS_TYPE_NVME,
        ] {
            assert_eq!(classify_evidence(&evidence(bus)), DriveLocality::Unproven);
        }
    }

    #[test]
    fn preliminary_drive_and_mapping_observations_are_never_sufficient() {
        let mut evidence = evidence(BUS_TYPE_NVME);
        evidence.dependency = QueryState::ApiFailure;
        assert_eq!(classify_evidence(&evidence), DriveLocality::Unknown);
    }

    #[test]
    fn every_virtual_or_fabric_dependency_fails_closed() {
        for (label, dependency) in [
            ("VHD", DependencyObservation::Vhd),
            ("VHDX", DependencyObservation::Vhdx),
            ("ISO", DependencyObservation::Iso),
            ("UNC-hosted VHD", DependencyObservation::UncHostedVhd),
        ] {
            let mut evidence = evidence(BUS_TYPE_NVME);
            evidence.dependency = QueryState::Complete(dependency);
            assert_eq!(
                classify_evidence(&evidence),
                DriveLocality::Unknown,
                "{label}"
            );
        }
    }

    #[test]
    fn unsupported_buses_are_cause_specific_unproven() {
        // WO29 Slice B: an unsupported bus on a completely-observed
        // topology is named `Unproven` with the unsupported-bus reason —
        // never a reasonless `Unknown`. The numeric bus value is still
        // recorded in the observed facts as an observation.
        for (label, bus) in [
            ("SCSI", BUS_TYPE_SCSI),
            ("Fibre", BUS_TYPE_FIBRE),
            ("RAID", BUS_TYPE_RAID),
            ("iSCSI", BUS_TYPE_ISCSI),
            ("SAS", BUS_TYPE_SAS),
            ("Virtual", BUS_TYPE_VIRTUAL),
            ("FileBackedVirtual", BUS_TYPE_FILE_BACKED_VIRTUAL),
            ("Spaces", BUS_TYPE_SPACES),
            ("NVMe-oF", BUS_TYPE_NVMEOF),
            ("future", 0xfeed),
        ] {
            let detail = classify_evidence_detail(&evidence(bus));
            assert_eq!(detail.locality, DriveLocality::Unproven, "{label}");
            assert_eq!(
                detail.unproven_reason,
                Some(REASON_UNSUPPORTED_BUS),
                "{label}"
            );
            assert_eq!(detail.backing_device_identity, vec![0], "{label}");
            assert!(
                !detail.observed_facts.is_empty(),
                "{label}: facts must be recorded, not dropped"
            );
            assert!(
                detail
                    .observed_facts
                    .iter()
                    .any(|fact| fact.contains(&format!("bus_type {bus} ("))),
                "{label}: {facts:?}",
                facts = detail.observed_facts
            );
        }
    }

    #[test]
    fn removable_backing_disk_is_cause_specific_unproven() {
        // WO29 Slice B: removable backing on a completely-observed
        // topology is named `Unproven` with the removable-media reason —
        // never a reasonless `Unknown`.
        let mut evidence = evidence(BUS_TYPE_SATA);
        make_removable(&mut evidence);
        let detail = classify_evidence_detail(&evidence);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.unproven_reason, Some(REASON_REMOVABLE_MEDIA));
        assert_eq!(detail.backing_device_identity, vec![0]);
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("removable_media true")),
            "{facts:?}",
            facts = detail.observed_facts
        );
    }

    #[test]
    fn multiple_complete_extents_and_disks_are_unproven() {
        let mut evidence = evidence(BUS_TYPE_ATA);
        evidence.extents = QueryState::Complete(vec![
            ExtentObservation {
                disk_number: 0,
                starting_offset: 0,
                extent_length: 4096,
            },
            ExtentObservation {
                disk_number: 1,
                starting_offset: 4096,
                extent_length: 8192,
            },
            ExtentObservation {
                disk_number: 0,
                starting_offset: 12_288,
                extent_length: 4096,
            },
        ]);
        evidence.disks = QueryState::Complete(vec![
            observed_record(0, false, BUS_TYPE_ATA),
            observed_record(1, false, BUS_TYPE_NVME),
        ]);
        let detail = classify_evidence_detail(&evidence);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.backing_device_identity, vec![0, 1]);
        assert_eq!(detail.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));
    }

    #[test]
    fn incomplete_or_inconsistent_extent_topology_fails_closed() {
        let cases = [
            vec![],
            vec![ExtentObservation {
                disk_number: 0,
                starting_offset: -1,
                extent_length: 4096,
            }],
            vec![ExtentObservation {
                disk_number: 0,
                starting_offset: 0,
                extent_length: 0,
            }],
        ];
        for extents in cases {
            let mut evidence = evidence(BUS_TYPE_NVME);
            evidence.extents = QueryState::Complete(extents);
            assert_eq!(classify_evidence(&evidence), DriveLocality::Unknown);
        }

        let mut missing = evidence(BUS_TYPE_NVME);
        missing.disks = QueryState::Complete(vec![]);
        assert_eq!(classify_evidence(&missing), DriveLocality::Unknown);

        let mut extra = evidence(BUS_TYPE_NVME);
        let QueryState::Complete(disks) = &mut extra.disks else {
            unreachable!();
        };
        disks.push(DiskQueryRecord {
            disk_number: 7,
            outcome: DiskQueryOutcome::Observed(DiskObservation {
                disk_number: 7,
                removable: false,
                bus_type: BUS_TYPE_NVME,
            }),
        });
        assert_eq!(classify_evidence(&extra), DriveLocality::Unknown);
    }

    #[test]
    fn incomplete_disk_record_membership_fails_closed_before_cause_selection() {
        // WO29 Item 2: complete disk-record membership is validated
        // BEFORE any cause is selected. A partial or contradictory record
        // set is never a complete observed matrix — neither case below
        // may reach the cause-specific `Unproven` paths, even though the
        // first record alone would name a cause.
        // Required [0,1], records [0(removable SD), 2(SD)]: disk 1 has no
        // record and disk 2 is extraneous. Must not pass as complete.
        let missing = multi_disk_evidence(
            &[0, 1],
            vec![
                observed_record(0, true, BUS_TYPE_SD),
                observed_record(2, false, BUS_TYPE_SD),
            ],
        );
        let detail = classify_evidence_detail(&missing);
        assert_eq!(detail.locality, DriveLocality::Unknown);
        assert_eq!(detail.unproven_reason, None);
        assert!(detail.backing_device_identity.is_empty());
        assert!(detail.observed_facts.is_empty());

        // Required [0,1], records [0(unsupported bus), 0(SD)]: duplicate
        // disk-0 records and no disk-1 record. The unsupported-bus cause
        // must not fire on a contradictory matrix.
        let contradictory = multi_disk_evidence(
            &[0, 1],
            vec![
                observed_record(0, false, BUS_TYPE_SCSI),
                observed_record(0, false, BUS_TYPE_SD),
            ],
        );
        let detail = classify_evidence_detail(&contradictory);
        assert_eq!(detail.locality, DriveLocality::Unknown);
        assert_eq!(detail.unproven_reason, None);

        // Single missing disk among three required: still closed.
        let one_missing = multi_disk_evidence(
            &[0, 1, 2],
            vec![
                observed_record(0, false, BUS_TYPE_ATA),
                observed_record(2, false, BUS_TYPE_SATA),
            ],
        );
        assert_eq!(classify_evidence(&one_missing), DriveLocality::Unknown);
    }

    #[test]
    fn valid_multi_disk_matrices_name_their_causes() {
        // WO29 Item 2: the corresponding VALID multi-disk controls —
        // complete record sets over the required disks reach the
        // cause-specific `Unproven` paths.
        // [0(SD), 1(MMC)], both non-removable observed buses:
        // insufficient-evidence (demoted grant-first).
        let observed_pair = multi_disk_evidence(
            &[0, 1],
            vec![
                observed_record(0, false, BUS_TYPE_SD),
                observed_record(1, false, BUS_TYPE_MMC),
            ],
        );
        let detail = classify_evidence_detail(&observed_pair);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));
        assert_eq!(detail.backing_device_identity, vec![0, 1]);

        // [0(removable SD), 1(ATA)]: removable-media cause.
        let removable_pair = multi_disk_evidence(
            &[0, 1],
            vec![
                observed_record(0, true, BUS_TYPE_SD),
                observed_record(1, false, BUS_TYPE_ATA),
            ],
        );
        let detail = classify_evidence_detail(&removable_pair);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.unproven_reason, Some(REASON_REMOVABLE_MEDIA));

        // [0(SCSI unsupported), 1(ATA)]: unsupported-bus cause.
        let unsupported_pair = multi_disk_evidence(
            &[0, 1],
            vec![
                observed_record(0, false, BUS_TYPE_SCSI),
                observed_record(1, false, BUS_TYPE_ATA),
            ],
        );
        let detail = classify_evidence_detail(&unsupported_pair);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.unproven_reason, Some(REASON_UNSUPPORTED_BUS));
    }

    #[test]
    fn partial_disk_query_outcomes_keep_available_facts_and_stay_unproven() {
        // WO29 Item 2: per-disk query outcomes distinguish failed,
        // undecodable, and not-attempted queries. The available
        // observations are preserved in the facts; the unavailable
        // determination is named `Unproven` with the unreadable reason —
        // never a reasonless `Unknown`, and never a claim that every
        // descriptor query failed.
        // Disk 0 observed (ATA), disk 1's query failed: the disk-0 bus
        // fact is kept and disk 1 is recorded as failed.
        let failed = multi_disk_evidence(
            &[0, 1],
            vec![
                observed_record(0, false, BUS_TYPE_ATA),
                DiskQueryRecord {
                    disk_number: 1,
                    outcome: DiskQueryOutcome::Failed,
                },
            ],
        );
        let detail = classify_evidence_detail(&failed);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(
            detail.unproven_reason,
            Some(REASON_REMOVABLE_STATUS_UNREADABLE)
        );
        assert_eq!(detail.backing_device_identity, vec![0, 1]);
        assert!(
            detail.observed_facts.iter().any(|fact| fact
                == "disk 0: bus_type 3 (ATA) removable_media false (query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive0)"),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("disk 1: descriptor query failed")),
            "{facts:?}",
            facts = detail.observed_facts
        );

        // Disk 0 failed, disk 1 never attempted: failed and
        // not-attempted stay distinct in the facts.
        let aborted = multi_disk_evidence(
            &[0, 1],
            vec![
                DiskQueryRecord {
                    disk_number: 0,
                    outcome: DiskQueryOutcome::Failed,
                },
                DiskQueryRecord {
                    disk_number: 1,
                    outcome: DiskQueryOutcome::NotAttempted,
                },
            ],
        );
        let detail = classify_evidence_detail(&aborted);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(
            detail.unproven_reason,
            Some(REASON_REMOVABLE_STATUS_UNREADABLE)
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("disk 0: descriptor query failed")),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("disk 1: descriptor query not attempted")),
            "{facts:?}",
            facts = detail.observed_facts
        );

        // Undecodable descriptor data is unavailable, not failed.
        let undecodable = multi_disk_evidence(
            &[0],
            vec![DiskQueryRecord {
                disk_number: 0,
                outcome: DiskQueryOutcome::Undecodable,
            }],
        );
        let detail = classify_evidence_detail(&undecodable);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(
            detail.unproven_reason,
            Some(REASON_REMOVABLE_STATUS_UNREADABLE)
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("disk 0: descriptor query undecodable")),
            "{facts:?}",
            facts = detail.observed_facts
        );
    }

    #[test]
    fn query_failures_and_partials_stay_closed_except_unreadable_disks() {
        for partial in [false, true] {
            let mut dependency = evidence(BUS_TYPE_NVME);
            dependency.dependency = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&dependency), DriveLocality::Unknown);

            let mut extents = evidence(BUS_TYPE_NVME);
            extents.extents = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&extents), DriveLocality::Unknown);

            // WO29 Item 2: an unreadable removable-status determination on
            // a completely-observed extent topology is named `Unproven`
            // with the unreadable reason — never a reasonless `Unknown`.
            let mut disks = evidence(BUS_TYPE_NVME);
            disks.disks = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            let detail = classify_evidence_detail(&disks);
            assert_eq!(detail.locality, DriveLocality::Unproven);
            assert_eq!(
                detail.unproven_reason,
                Some(REASON_REMOVABLE_STATUS_UNREADABLE)
            );

            let mut before = evidence(BUS_TYPE_NVME);
            before.before.mapping = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&before), DriveLocality::Unknown);

            let mut after = evidence(BUS_TYPE_NVME);
            after.after.mapping = if partial {
                QueryState::Partial
            } else {
                QueryState::ApiFailure
            };
            assert_eq!(classify_evidence(&after), DriveLocality::Unknown);
        }
    }

    #[test]
    fn unreadable_removable_status_records_unavailable_facts() {
        // WO29 Item 2: the extent topology is completely observed but the
        // removable-status query failed, so the disk facts are recorded as
        // unavailable — never silently dropped.
        let mut evidence = evidence(BUS_TYPE_NVME);
        evidence.disks = QueryState::ApiFailure;
        let detail = classify_evidence_detail(&evidence);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(
            detail.unproven_reason,
            Some(REASON_REMOVABLE_STATUS_UNREADABLE)
        );
        // The extent observation is kept; only the disk determination is
        // unavailable.
        assert_eq!(detail.backing_device_identity, vec![0]);
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact == "dependency_walk: no_dependencies"),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("extent: disk 0")),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            detail.observed_facts.iter().any(|fact| fact
                == "disk 0: bus_type unavailable, removable_media unavailable (query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia unreadable via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive0)"),
            "{facts:?}",
            facts = detail.observed_facts
        );
    }

    #[test]
    fn topology_recheck_must_be_identical() {
        let mut mapping_changed = evidence(BUS_TYPE_NVME);
        mapping_changed.after.mapping = mapping(r"\Device\HarddiskVolume4");
        assert_eq!(classify_evidence(&mapping_changed), DriveLocality::Unknown);

        let mut drive_changed = evidence(BUS_TYPE_NVME);
        drive_changed.after.drive_type = DriveTypeObservation::Remote;
        assert_eq!(classify_evidence(&drive_changed), DriveLocality::Unknown);
    }

    #[test]
    fn cleanup_failure_preserves_observations_with_named_cleanup_fact() {
        // WO29 Slice B correction: a failed CloseHandle is cleanup
        // failure, not evidence failure. The reducer no longer wipes the
        // observations — the disk observations are preserved, the
        // cleanup failure is carried as its own named fact, and the
        // verdict follows the observed matrix (grant-first
        // insufficient-evidence for this NVMe fixture) instead of
        // collapsing to reasonless `Unknown`.
        let mut evidence = evidence(BUS_TYPE_NVME);
        evidence.closes = QueryState::ApiFailure;
        let detail = classify_evidence_detail(&evidence);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));
        assert_eq!(detail.backing_device_identity, vec![0]);
        assert!(
            detail.observed_facts.iter().any(|fact| fact
                == "disk 0: bus_type 17 (NVMe) removable_media false (query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive0)"),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.starts_with("cleanup: close_failed")),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(!detail.contradiction);
    }

    #[test]
    fn open_failure_is_not_a_descriptor_query_failure() {
        // WO29 Slice B correction: a device that never opened is not a
        // descriptor query that ran and failed. The producer records
        // `OpenFailed` (never `Failed`), the fact names the open failure
        // and explicitly states no query ran, and the unavailable
        // determination is named `Unproven` with the unreadable reason —
        // never a claim that every descriptor query failed.
        let open_failed = multi_disk_evidence(
            &[0, 1],
            vec![
                DiskQueryRecord {
                    disk_number: 0,
                    outcome: DiskQueryOutcome::OpenFailed,
                },
                DiskQueryRecord {
                    disk_number: 1,
                    outcome: DiskQueryOutcome::NotAttempted,
                },
            ],
        );
        let detail = classify_evidence_detail(&open_failed);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(
            detail.unproven_reason,
            Some(REASON_REMOVABLE_STATUS_UNREADABLE)
        );
        assert!(
            detail.observed_facts.iter().any(|fact| fact
                == "disk 0: device open failed (no handle to PhysicalDrive0); descriptor query never ran; bus_type unavailable, removable_media unavailable"),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            !detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("disk 0: descriptor query failed")),
            "an open failure must never claim a descriptor query failed: {facts:?}",
            facts = detail.observed_facts
        );
        assert!(!detail.contradiction);
    }

    #[test]
    fn before_after_contradiction_carries_marker_to_binder() {
        // WO29 Slice B correction: a known before/after mismatch is
        // contradictory evidence, never missing evidence. The locality
        // verdict stays closed (`Unknown` — nothing admits), but the
        // contradiction marker and the before/after facts are preserved
        // so the runtime external-trust binder can refuse them before
        // payload consumption. A non-contradictory `Unknown` (missing
        // dependency evidence) carries no marker.
        let mut changed = evidence(BUS_TYPE_NVME);
        changed.after.mapping = mapping(r"\Device\HarddiskVolume4");
        assert_eq!(classify_evidence(&changed), DriveLocality::Unknown);
        let detail = classify_evidence_detail(&changed);
        assert_eq!(detail.locality, DriveLocality::Unknown);
        assert!(detail.contradiction);
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("equality: mismatch")),
            "{facts:?}",
            facts = detail.observed_facts
        );
        assert!(
            detail
                .observed_facts
                .iter()
                .any(|fact| fact.contains("known before/after contradiction")),
            "{facts:?}",
            facts = detail.observed_facts
        );

        // After stopped being a candidate: same marker.
        let mut recandidate = evidence(BUS_TYPE_NVME);
        recandidate.after.drive_type = DriveTypeObservation::Remote;
        let detail = classify_evidence_detail(&recandidate);
        assert_eq!(detail.locality, DriveLocality::Unknown);
        assert!(detail.contradiction);

        // Missing evidence is not contradiction: a dependency failure is
        // still `Unknown`, with no marker and no preserved facts.
        let mut missing = evidence(BUS_TYPE_NVME);
        missing.dependency = QueryState::ApiFailure;
        let detail = classify_evidence_detail(&missing);
        assert_eq!(detail.locality, DriveLocality::Unknown);
        assert!(!detail.contradiction);
        assert!(detail.observed_facts.is_empty());
    }

    #[test]
    fn inner_disk_identity_mismatch_rejects_before_cause_selection() {
        // WO29 Slice B correction: the reducer validates the inner
        // identity inside an `Observed` record, not just the outer
        // record membership. Required disk 0 with an outer record for
        // disk 0 whose inner observation names disk 99 fails closed
        // before any cause is selected — it can never reach the
        // cause-specific `Unproven` paths even though the outer numbers
        // line up.
        let mismatch = multi_disk_evidence(
            &[0],
            vec![DiskQueryRecord {
                disk_number: 0,
                outcome: DiskQueryOutcome::Observed(DiskObservation {
                    disk_number: 99,
                    removable: false,
                    bus_type: BUS_TYPE_NVME,
                }),
            }],
        );
        let detail = classify_evidence_detail(&mismatch);
        assert_eq!(detail.locality, DriveLocality::Unknown);
        assert_eq!(detail.unproven_reason, None);
        assert!(!detail.contradiction);

        // Valid control: matching inner identity proceeds to the cause
        // matrix (grant-first insufficient-evidence for this NVMe
        // fixture).
        let valid = multi_disk_evidence(&[0], vec![observed_record(0, false, BUS_TYPE_NVME)]);
        let detail = classify_evidence_detail(&valid);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));
    }

    #[test]
    fn preliminary_remote_substituted_removable_and_unknown_results_remain_closed() {
        for (drive_type, target, expected) in [
            (
                DriveTypeObservation::Fixed,
                r"\Device\Mup\server\share",
                DriveLocality::Remote,
            ),
            (
                DriveTypeObservation::Fixed,
                r"\??\C:\workspace",
                DriveLocality::Substituted,
            ),
            (
                DriveTypeObservation::Removable,
                r"\Device\HarddiskVolume3",
                DriveLocality::Removable,
            ),
            (
                DriveTypeObservation::Optical,
                r"\Device\HarddiskVolume3",
                DriveLocality::Removable,
            ),
            (
                DriveTypeObservation::RamDisk,
                r"\Device\HarddiskVolume3",
                DriveLocality::Removable,
            ),
            (
                DriveTypeObservation::Unknown,
                r"\Device\HarddiskVolume3",
                DriveLocality::Unknown,
            ),
            (
                DriveTypeObservation::MissingRoot,
                r"\Device\HarddiskVolume3",
                DriveLocality::Unknown,
            ),
            (
                DriveTypeObservation::Other,
                r"\Device\HarddiskVolume3",
                DriveLocality::Unknown,
            ),
        ] {
            let mut evidence = evidence(BUS_TYPE_NVME);
            evidence.before = PreliminaryObservation {
                drive_type,
                mapping: mapping(target),
            };
            assert_eq!(classify_evidence(&evidence), expected);
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_abi_layouts_and_buffer_decoders_are_bounded() {
        use super::{
            DiskExtentLayout, QueryState, StorageDependencyInfoType2Layout,
            StorageDeviceDescriptorLayout, VolumeDiskExtentsLayout, decode_device_mapping,
        };
        assert_eq!(core::mem::size_of::<DiskExtentLayout>(), 24);
        assert_eq!(core::mem::offset_of!(VolumeDiskExtentsLayout, extent), 8);
        assert_eq!(core::mem::size_of::<StorageDependencyInfoType2Layout>(), 64);
        assert_eq!(
            core::mem::offset_of!(StorageDeviceDescriptorLayout, bus_type),
            28
        );
        assert_eq!(core::mem::size_of::<StorageDeviceDescriptorLayout>(), 36);

        assert_eq!(decode_device_mapping(&[0; 4], 0), QueryState::ApiFailure);
        assert_eq!(decode_device_mapping(&[0; 4], 5), QueryState::Partial);
        assert_eq!(
            decode_device_mapping(&[u16::from(b'x'); 4], 4),
            QueryState::Partial
        );
        assert_eq!(
            decode_device_mapping(&[u16::from(b'x'), 0, u16::from(b'y'), 0], 4),
            QueryState::Partial
        );
    }

    #[cfg(windows)]
    #[test]
    fn dependency_decoder_rejects_api_and_variable_length_boundary_failures() {
        use super::{
            DependencyObservation, QueryState, StorageDependencyInfoType2Layout,
            decode_dependency_information,
        };
        let entry_bytes = core::mem::size_of::<StorageDependencyInfoType2Layout>();
        let mut none = vec![0u8; 8];
        put_u32(&mut none, 0, 2);
        assert_eq!(
            decode_dependency_information(&none, 0, 8),
            QueryState::Complete(DependencyObservation::None)
        );
        assert_eq!(
            decode_dependency_information(&none, 5, 8),
            QueryState::ApiFailure
        );
        assert_eq!(
            decode_dependency_information(&none[..7], 0, 7),
            QueryState::Partial
        );
        assert_eq!(
            decode_dependency_information(&none, 0, 9),
            QueryState::Partial
        );

        let mut wrong_version = none.clone();
        put_u32(&mut wrong_version, 0, 1);
        assert_eq!(
            decode_dependency_information(&wrong_version, 0, 8),
            QueryState::Partial
        );

        let mut wrong_count = none.clone();
        put_u32(&mut wrong_count, 4, 1);
        assert_eq!(
            decode_dependency_information(&wrong_count, 0, 8),
            QueryState::Partial
        );
        put_u32(&mut wrong_count, 4, 1025);
        assert_eq!(
            decode_dependency_information(&wrong_count, 0, 8),
            QueryState::Partial
        );

        let mut present = vec![0u8; 8 + entry_bytes];
        put_u32(&mut present, 0, 2);
        put_u32(&mut present, 4, 1);
        assert_eq!(
            decode_dependency_information(&present, 0, present.len() as u32),
            QueryState::Complete(DependencyObservation::Present)
        );
    }

    #[cfg(windows)]
    #[test]
    fn extent_decoder_rejects_api_truncation_oversize_and_inconsistent_count() {
        use super::{
            DiskExtentLayout, QueryState, VolumeDiskExtentsLayout, decode_extent_information,
        };
        let base = core::mem::offset_of!(VolumeDiskExtentsLayout, extent);
        let extent_size = core::mem::size_of::<DiskExtentLayout>();
        let start = core::mem::offset_of!(DiskExtentLayout, starting_offset);
        let length = core::mem::offset_of!(DiskExtentLayout, extent_length);
        let mut valid = vec![0u8; base + extent_size];
        put_u32(&mut valid, 0, 1);
        put_u32(&mut valid, base, 7);
        put_i64(&mut valid, base + start, 4096);
        put_i64(&mut valid, base + length, 8192);
        let decoded = decode_extent_information(&valid, Some(valid.len()));
        assert!(matches!(decoded, QueryState::Complete(ref rows) if rows.len() == 1));
        assert_eq!(
            decode_extent_information(&valid, None),
            QueryState::ApiFailure
        );
        assert_eq!(
            decode_extent_information(&valid, Some(3)),
            QueryState::Partial
        );
        assert_eq!(
            decode_extent_information(&valid, Some(valid.len() + 1)),
            QueryState::Partial
        );

        let mut zero = valid.clone();
        put_u32(&mut zero, 0, 0);
        assert_eq!(
            decode_extent_information(&zero, Some(zero.len())),
            QueryState::Partial
        );
        let mut inconsistent = valid.clone();
        put_u32(&mut inconsistent, 0, 2);
        assert_eq!(
            decode_extent_information(&inconsistent, Some(inconsistent.len())),
            QueryState::Partial
        );
    }

    #[cfg(windows)]
    #[test]
    fn descriptor_decoder_rejects_api_wrong_version_and_inconsistent_lengths() {
        use super::{
            BUS_TYPE_NVME, QueryState, StorageDeviceDescriptorLayout, decode_disk_descriptor,
        };
        let minimum = core::mem::size_of::<StorageDeviceDescriptorLayout>();
        let bus = core::mem::offset_of!(StorageDeviceDescriptorLayout, bus_type);
        let removable = core::mem::offset_of!(StorageDeviceDescriptorLayout, removable_media);
        let mut valid = vec![0u8; minimum];
        put_u32(&mut valid, 0, minimum as u32);
        put_u32(&mut valid, 4, minimum as u32);
        put_u32(&mut valid, bus, BUS_TYPE_NVME);
        valid[removable] = 0;
        assert!(matches!(
            decode_disk_descriptor(&valid, Some(valid.len()), 4),
            QueryState::Complete(_)
        ));
        assert_eq!(
            decode_disk_descriptor(&valid, None, 4),
            QueryState::ApiFailure
        );
        assert_eq!(
            decode_disk_descriptor(&valid, Some(minimum - 1), 4),
            QueryState::Partial
        );
        assert_eq!(
            decode_disk_descriptor(&valid, Some(minimum + 1), 4),
            QueryState::Partial
        );

        let mut wrong_version = valid.clone();
        put_u32(&mut wrong_version, 0, (minimum - 1) as u32);
        assert_eq!(
            decode_disk_descriptor(&wrong_version, Some(minimum), 4),
            QueryState::Partial
        );
        let mut wrong_size = valid.clone();
        put_u32(&mut wrong_size, 4, (minimum + 1) as u32);
        assert_eq!(
            decode_disk_descriptor(&wrong_size, Some(minimum), 4),
            QueryState::Partial
        );
        put_u32(&mut wrong_size, 4, (minimum - 1) as u32);
        assert_eq!(
            decode_disk_descriptor(&wrong_size, Some(minimum), 4),
            QueryState::Partial
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_classification_is_unsupported_without_foreign_calls() {
        let root = DriveRoot::from_ascii_letter(b'C').expect("drive root");
        assert_eq!(
            super::classify(root),
            ClassifiedDrive {
                locality: DriveLocality::Unsupported,
                backing_device_identity: Vec::new(),
                observed_facts: Vec::new(),
                unproven_reason: None,
                volume_serial: None,
                contradiction: false,
            }
        );
    }

    #[test]
    fn evidence_detail_threads_sorted_unique_backing_disk_numbers() {
        let mut detail_evidence = evidence(BUS_TYPE_ATA);
        detail_evidence.extents = QueryState::Complete(vec![
            ExtentObservation {
                disk_number: 2,
                starting_offset: 0,
                extent_length: 4096,
            },
            ExtentObservation {
                disk_number: 0,
                starting_offset: 4096,
                extent_length: 4096,
            },
            ExtentObservation {
                disk_number: 2,
                starting_offset: 8192,
                extent_length: 4096,
            },
            ExtentObservation {
                disk_number: 1,
                starting_offset: 12_288,
                extent_length: 4096,
            },
        ]);
        detail_evidence.disks = QueryState::Complete(vec![
            observed_record(2, false, BUS_TYPE_NVME),
            observed_record(0, false, BUS_TYPE_ATA),
            observed_record(1, false, BUS_TYPE_SATA),
        ]);
        let detail = classify_evidence_detail(&detail_evidence);
        assert_eq!(detail.locality, DriveLocality::Unproven);
        assert_eq!(detail.backing_device_identity, vec![0, 1, 2]);
        assert_eq!(detail.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));
        // One observed-fact line per required disk, in disk-number order.
        let disk_facts: Vec<&String> = detail
            .observed_facts
            .iter()
            .filter(|fact| fact.starts_with("disk "))
            .collect();
        assert_eq!(disk_facts.len(), 3);
        assert!(disk_facts[0].starts_with("disk 0:"));
        assert!(disk_facts[1].starts_with("disk 1:"));
        assert!(disk_facts[2].starts_with("disk 2:"));
    }

    #[test]
    fn demoted_path_carries_observed_identity_closed_paths_carry_none() {
        let demoted = classify_evidence_detail(&evidence(BUS_TYPE_NVME));
        assert_eq!(demoted.locality, DriveLocality::Unproven);
        assert_eq!(demoted.backing_device_identity, vec![0]);
        assert!(!demoted.observed_facts.is_empty());
        assert_eq!(demoted.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));

        // Cause-specific paths are observed paths too: identity, facts,
        // and reason are carried.
        let mut removable = evidence(BUS_TYPE_SATA);
        make_removable(&mut removable);
        let removable = classify_evidence_detail(&removable);
        assert_eq!(removable.locality, DriveLocality::Unproven);
        assert_eq!(removable.backing_device_identity, vec![0]);
        assert!(!removable.observed_facts.is_empty());
        assert_eq!(removable.unproven_reason, Some(REASON_REMOVABLE_MEDIA));

        let mut unknown = evidence(BUS_TYPE_NVME);
        unknown.dependency = QueryState::ApiFailure;
        let unknown = classify_evidence_detail(&unknown);
        assert_eq!(unknown.locality, DriveLocality::Unknown);
        assert!(unknown.backing_device_identity.is_empty());
        assert!(unknown.observed_facts.is_empty());
        assert_eq!(unknown.unproven_reason, None);

        let mut remote = evidence(BUS_TYPE_NVME);
        remote.before = PreliminaryObservation {
            drive_type: DriveTypeObservation::Fixed,
            mapping: mapping(r"\Device\Mup\server\share"),
        };
        let remote = classify_evidence_detail(&remote);
        assert_eq!(remote.locality, DriveLocality::Remote);
        assert!(remote.backing_device_identity.is_empty());
        assert!(remote.observed_facts.is_empty());
        assert_eq!(remote.unproven_reason, None);
    }

    #[test]
    fn unproven_reason_is_set_on_observed_paths_never_on_closed() {
        // Every observed-but-unproven path carries its named reason.
        for (label, evidence, reason) in [
            (
                "demoted",
                evidence(BUS_TYPE_ATA),
                REASON_INSUFFICIENT_EVIDENCE,
            ),
            (
                "sd-observed",
                evidence(BUS_TYPE_SD),
                REASON_INSUFFICIENT_EVIDENCE,
            ),
            (
                "mmc-observed",
                evidence(BUS_TYPE_MMC),
                REASON_INSUFFICIENT_EVIDENCE,
            ),
        ] {
            let detail = classify_evidence_detail(&evidence);
            assert_eq!(detail.locality, DriveLocality::Unproven, "{label}");
            assert_eq!(detail.unproven_reason, Some(reason), "{label}");
            assert!(!detail.observed_facts.is_empty(), "{label}");
        }

        let mut removable = evidence(BUS_TYPE_SATA);
        make_removable(&mut removable);
        let removable = classify_evidence_detail(&removable);
        assert_eq!(removable.unproven_reason, Some(REASON_REMOVABLE_MEDIA));

        let unsupported = classify_evidence_detail(&evidence(BUS_TYPE_SCSI));
        assert_eq!(unsupported.unproven_reason, Some(REASON_UNSUPPORTED_BUS));

        let mut unreadable = evidence(BUS_TYPE_NVME);
        unreadable.disks = QueryState::ApiFailure;
        let unreadable = classify_evidence_detail(&unreadable);
        assert_eq!(
            unreadable.unproven_reason,
            Some(REASON_REMOVABLE_STATUS_UNREADABLE)
        );

        // Closed paths carry no reason and no facts.
        let mut substituted = evidence(BUS_TYPE_NVME);
        substituted.before.mapping = mapping(r"\??\C:\workspace");
        let mut removable_drive = evidence(BUS_TYPE_NVME);
        removable_drive.before.drive_type = DriveTypeObservation::Removable;
        let mut unknown = evidence(BUS_TYPE_NVME);
        unknown.extents = QueryState::ApiFailure;
        for fixture in [&substituted, &removable_drive, &unknown] {
            let detail = classify_evidence_detail(fixture);
            assert_ne!(detail.locality, DriveLocality::Unproven);
            assert_eq!(detail.unproven_reason, None, "{:?}", detail.locality);
            assert!(detail.observed_facts.is_empty());
            assert!(detail.backing_device_identity.is_empty());
        }
    }

    #[test]
    fn sd_mmc_removable_yields_removable_media_reason() {
        // WO29 Item 2: an SD/MMC disk with RemovableMedia set is observed
        // but not admitted — the removable-media cause is named.
        for (label, bus) in [("SD", BUS_TYPE_SD), ("MMC", BUS_TYPE_MMC)] {
            let mut evidence = evidence(bus);
            make_removable(&mut evidence);
            let detail = classify_evidence_detail(&evidence);
            assert_eq!(detail.locality, DriveLocality::Unproven, "{label}");
            assert_eq!(
                detail.unproven_reason,
                Some(REASON_REMOVABLE_MEDIA),
                "{label}"
            );
        }
    }

    #[test]
    fn evidence_bundle_carries_the_complete_observed_fact_record() {
        // WO29 Slice B: the observed-fact record carries the complete
        // mandated surface — numeric/name bus facts, removable value and
        // query provenance, before/after observations and equality, the
        // configured observed-bus list, the P1–P4 mapping, and the
        // classification with provenance.
        let detail = classify_evidence_detail(&evidence(BUS_TYPE_SD));
        assert_eq!(detail.locality, DriveLocality::Unproven);
        let facts = &detail.observed_facts;
        let has = |needle: &str| facts.iter().any(|fact| fact.contains(needle));
        // dependency-walk observation and the extent observation
        assert!(has("dependency_walk: no_dependencies"), "{facts:?}");
        assert!(
            has("extent: disk 0 starting_offset 1048576 extent_length 4194304"),
            "{facts:?}"
        );
        // numeric/name bus fact with removable value and query provenance
        assert!(
            has("disk 0: bus_type 12 (SD) removable_media false"),
            "{facts:?}"
        );
        assert!(
            has(
                "query: STORAGE_DEVICE_DESCRIPTOR.RemovableMedia via IOCTL_STORAGE_QUERY_PROPERTY on PhysicalDrive0"
            ),
            "{facts:?}"
        );
        // before/after observations and equality
        assert!(has("before: drive_type Fixed"), "{facts:?}");
        assert!(has("after: drive_type Fixed"), "{facts:?}");
        assert!(has("equality: match"), "{facts:?}");
        // configured observed-bus list
        assert!(
            has("observed_bus_list: ATA(3), SATA(11), SD(12), MMC(13), NVMe(17)"),
            "{facts:?}"
        );
        // one-line P1–P4 mapping with the named reason
        assert!(
            has("P1-P4: P1 unproven (windows_locality_unproven_insufficient_evidence_v0)"),
            "{facts:?}"
        );
        // classification with provenance
        assert!(
            has("classification: Unproven (observed-but-unproven"),
            "{facts:?}"
        );
        assert!(has("grant-first per decision 0029 §14"), "{facts:?}");
        // The reason is named and no fact claims proof.
        assert_eq!(detail.unproven_reason, Some(REASON_INSUFFICIENT_EVIDENCE));
        assert!(
            facts
                .iter()
                .all(|fact| !fact.contains("proved") || fact.contains("unproven")),
            "{facts:?}"
        );
    }

    #[test]
    fn no_classifier_path_emits_fixed_local() {
        let mut fixtures = Vec::new();
        for bus in [
            BUS_TYPE_ATA,
            BUS_TYPE_SATA,
            BUS_TYPE_SD,
            BUS_TYPE_MMC,
            BUS_TYPE_NVME,
            BUS_TYPE_SCSI,
            BUS_TYPE_FIBRE,
            BUS_TYPE_RAID,
            BUS_TYPE_ISCSI,
            BUS_TYPE_SAS,
            BUS_TYPE_VIRTUAL,
            BUS_TYPE_FILE_BACKED_VIRTUAL,
            BUS_TYPE_SPACES,
            BUS_TYPE_NVMEOF,
            0xfeed,
        ] {
            fixtures.push(evidence(bus));
        }
        let mut dependency_failed = evidence(BUS_TYPE_NVME);
        dependency_failed.dependency = QueryState::ApiFailure;
        fixtures.push(dependency_failed);
        let mut closes_failed = evidence(BUS_TYPE_NVME);
        closes_failed.closes = QueryState::ApiFailure;
        fixtures.push(closes_failed);
        let mut topology_changed = evidence(BUS_TYPE_NVME);
        topology_changed.after.mapping = mapping(r"\Device\HarddiskVolume4");
        fixtures.push(topology_changed);
        let mut removable = evidence(BUS_TYPE_SATA);
        make_removable(&mut removable);
        fixtures.push(removable);
        let mut preliminary_closed = evidence(BUS_TYPE_NVME);
        preliminary_closed.before = PreliminaryObservation {
            drive_type: DriveTypeObservation::Fixed,
            mapping: mapping(r"\??\C:\workspace"),
        };
        fixtures.push(preliminary_closed);

        for fixture in &fixtures {
            assert_ne!(
                classify_evidence(fixture),
                DriveLocality::FixedLocal,
                "classify_evidence emitted FixedLocal"
            );
            assert_ne!(
                classify_evidence_detail(fixture).locality,
                DriveLocality::FixedLocal,
                "classify_evidence_detail emitted FixedLocal"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn file_id_info_layout_matches_windows_abi() {
        use super::FileIdInfoLayout;
        use core::mem::{offset_of, size_of};
        // `FILE_ID_INFO`: ULONGLONG VolumeSerialNumber followed by the
        // 128-bit FILE_ID_128 — 24 bytes on every target.
        assert_eq!(size_of::<FileIdInfoLayout>(), 24);
        assert_eq!(offset_of!(FileIdInfoLayout, volume_serial_number), 0);
        assert_eq!(offset_of!(FileIdInfoLayout, file_id), 8);
    }

    #[cfg(windows)]
    #[test]
    fn opened_file_identity_rejects_null_handle() {
        assert_eq!(super::opened_file_identity(core::ptr::null_mut()), None);
    }

    #[cfg(windows)]
    #[test]
    fn opened_file_identity_is_stable_across_handles_to_the_same_file() {
        use std::os::windows::io::AsRawHandle;
        let path = std::env::temp_dir().join("wo29-slice-a-leaf-d-file-identity-probe.tmp");
        let first = std::fs::File::create(&path).expect("create probe file");
        let identity =
            super::opened_file_identity(first.as_raw_handle()).expect("identity for open handle");
        let second = std::fs::OpenOptions::new()
            .read(true)
            .open(&path)
            .expect("reopen probe file");
        let reopened = super::opened_file_identity(second.as_raw_handle())
            .expect("identity for reopened handle");
        assert_eq!(identity, reopened);
        drop(first);
        drop(second);
        std::fs::remove_file(&path).ok();
    }

    /// Volume-GUID final path fixtures, built without literal backslashes
    /// (repo text-hygiene rule). The real `GetFinalPathNameByHandleW`
    /// `VOLUME_NAME_GUID` form names the full file path inside the volume
    /// — the volume GUID followed by the in-volume path — which is the
    /// form the opened-handle producer actually returns. The root-only
    /// fixture covers the degenerate input the old root-only parser was
    /// written against; production never emits it.
    fn guid_final_file_path_fixture() -> Vec<u16> {
        let bs = char::from(92);
        format!("{bs}{bs}?{bs}Volume{{b75e2c83-0000-0000-0000-602f00000000}}{bs}opaque{bs}file.bin")
            .encode_utf16()
            .collect()
    }

    fn guid_final_root_path_fixture() -> Vec<u16> {
        let bs = char::from(92);
        format!("{bs}{bs}?{bs}Volume{{b75e2c83-0000-0000-0000-602f00000000}}{bs}")
            .encode_utf16()
            .collect()
    }

    fn guid_root_device_fixture() -> Vec<u16> {
        let bs = char::from(92);
        let mut device: Vec<u16> =
            format!("{bs}{bs}?{bs}Volume{{b75e2c83-0000-0000-0000-602f00000000}}")
                .encode_utf16()
                .collect();
        device.push(0);
        device
    }

    #[test]
    fn containing_volume_device_path_extracts_root_from_full_file_path() {
        // The real producer's form: a full GUID file path reduces to the
        // containing volume's device path, dropping the in-volume suffix.
        let device = super::containing_volume_device_path(&guid_final_file_path_fixture())
            .expect("device path");
        assert_eq!(device, guid_root_device_fixture());
        assert_eq!(device.last(), Some(&0));
    }

    #[test]
    fn containing_volume_device_path_accepts_root_only_form() {
        let device = super::containing_volume_device_path(&guid_final_root_path_fixture())
            .expect("device path");
        assert_eq!(device, guid_root_device_fixture());
        assert_eq!(device.last(), Some(&0));
    }

    #[test]
    fn containing_volume_device_path_rejects_dos_letter_paths() {
        // A drive-letter final path is not a volume-GUID form: the
        // device path builder must not re-derive a letter device.
        let bs = char::from(92);
        let dos_path: Vec<u16> = format!("{bs}{bs}?{bs}C:{bs}Windows{bs}")
            .encode_utf16()
            .collect();
        assert_eq!(super::containing_volume_device_path(&dos_path), None);
    }

    #[test]
    fn containing_volume_device_path_rejects_missing_closing_brace() {
        let bs = char::from(92);
        let unclosed: Vec<u16> =
            format!("{bs}{bs}?{bs}Volume{{b75e2c83-0000-0000-0000-602f00000000{bs}file.bin")
                .encode_utf16()
                .collect();
        assert_eq!(super::containing_volume_device_path(&unclosed), None);
    }

    #[test]
    fn containing_volume_device_path_rejects_empty_and_unc_paths() {
        assert_eq!(super::containing_volume_device_path(&[]), None);
        let bs = char::from(92);
        let unc_path: Vec<u16> = format!("{bs}{bs}host{bs}share{bs}")
            .encode_utf16()
            .collect();
        assert_eq!(super::containing_volume_device_path(&unc_path), None);
    }

    /// Production-connected control: the opened-file → containing-volume
    /// → serial/extents chain on a REAL handle. This traverses the actual
    /// producers — `GetFinalPathNameByHandleW` GUID final path, GUID-root
    /// extraction, volume-device open, `FILE_ID_INFO` serial on the opened
    /// file handle, the disk-extent ioctl — which a synthetic
    /// `OpenedVolumeObservation` fixture cannot prove. The observation's
    /// serial must equal the opened file identity's serial from the SAME
    /// handle, and the extent set must be complete (non-empty, sorted,
    /// deduplicated).
    #[cfg(windows)]
    #[test]
    fn opened_volume_observation_traverses_real_producers_coherently() {
        use std::os::windows::io::AsRawHandle;

        let path = std::env::temp_dir().join("wo29-slice-a-volume-observation-probe.tmp");
        std::fs::write(&path, b"probe").expect("create probe file");
        let file = std::fs::File::open(&path).expect("open probe file");
        let handle = file.as_raw_handle();
        let identity = super::opened_file_identity(handle).expect("identity for open handle");
        let observation = super::opened_volume_observation(handle)
            .expect("volume observation for a real opened file");
        assert_eq!(
            observation.volume_serial, identity.volume_serial,
            "the observation serial and the file identity serial name the same volume"
        );
        assert!(
            !observation.disk_numbers.is_empty(),
            "the observation carries the complete extent set"
        );
        let mut sorted = observation.disk_numbers.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            observation.disk_numbers, sorted,
            "disk numbers are sorted and deduplicated"
        );
        drop(file);
        std::fs::remove_file(&path).ok();
    }
}

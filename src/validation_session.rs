//! Private Work Order 11 validation-session implementation.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{
    CliOptions, CommandObservation, LoadedProgram, LoadedSource, Program, callable,
    load_program_from_sources, observe_loaded_program, parse_cli, predicate, read_program_sources,
    typed_failure,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValidationPolicy {
    Reference,
    Optimized,
}

impl ValidationPolicy {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Optimized => "optimized",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CacheEligibility {
    EligibleStatic,
    MandatoryFresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ObservationRequest {
    pub(super) conclusion_id: &'static str,
    pub(super) arguments: Vec<String>,
    pub(super) cache_eligibility: CacheEligibility,
}

impl ObservationRequest {
    pub(super) fn static_observation(
        conclusion_id: &'static str,
        arguments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            conclusion_id,
            arguments: arguments.into_iter().map(Into::into).collect(),
            cache_eligibility: CacheEligibility::EligibleStatic,
        }
    }

    pub(super) fn fresh_observation(
        conclusion_id: &'static str,
        arguments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            conclusion_id,
            arguments: arguments.into_iter().map(Into::into).collect(),
            cache_eligibility: CacheEligibility::MandatoryFresh,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SessionIdentity {
    ordered_paths: Vec<PathBuf>,
    source_bytes: Vec<Vec<u8>>,
    semantic_environment: String,
}

impl SessionIdentity {
    fn from_sources(sources: &[LoadedSource]) -> Self {
        Self {
            ordered_paths: sources.iter().map(|source| source.path.clone()).collect(),
            source_bytes: sources.iter().map(|source| source.bytes.clone()).collect(),
            semantic_environment: semantic_environment_identity(),
        }
    }

    pub(super) fn ordered_paths(&self) -> &[PathBuf] {
        &self.ordered_paths
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ObservationCacheKey {
    session: SessionIdentity,
    arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct SourceSnapshot {
    pub(super) path: PathBuf,
    pub(super) bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SessionPhaseDurations {
    pub(super) source_read: Duration,
    pub(super) parse: Duration,
    pub(super) initial_check: Duration,
    pub(super) substantive_static_analysis: Duration,
    pub(super) cache_lookup: Duration,
    pub(super) cache_hit: Duration,
    pub(super) cache_miss: Duration,
    pub(super) cache_entry_construction: Duration,
    pub(super) cache_reuse: Duration,
    pub(super) fresh_runtime_execution: Duration,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SessionMetrics {
    pub(super) distinct_source_identities: u64,
    pub(super) source_reads: u64,
    pub(super) parses: u64,
    pub(super) initial_checks: u64,
    pub(super) static_requests: u64,
    pub(super) cacheable_requests: u64,
    pub(super) non_cacheable_requests: u64,
    pub(super) substantive_static_computations: u64,
    pub(super) cache_lookups: u64,
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) cache_entries_constructed: u64,
    pub(super) cache_reuses: u64,
    pub(super) runtime_requests: u64,
    pub(super) runtime_executions: u64,
    pub(super) runtime_result_cache_hits: u64,
    pub(super) phases: SessionPhaseDurations,
}

impl SessionMetrics {
    pub(super) fn assert_equations(&self) -> Result<(), String> {
        if self.static_requests
            != self
                .substantive_static_computations
                .checked_add(self.cache_hits)
                .ok_or("static request equation overflow")?
        {
            return Err("static requests must equal computations plus cache hits".to_string());
        }
        if self.static_requests
            != self
                .cacheable_requests
                .checked_add(self.non_cacheable_requests)
                .ok_or("static cacheability equation overflow")?
        {
            return Err(
                "static requests must equal cacheable plus non-cacheable requests".to_string(),
            );
        }
        if self.cacheable_requests != self.cache_lookups {
            return Err("cacheable requests must equal cache lookups".to_string());
        }
        if self.cache_lookups
            != self
                .cache_hits
                .checked_add(self.cache_misses)
                .ok_or("cache lookup equation overflow")?
        {
            return Err("cache lookups must equal hits plus misses".to_string());
        }
        if self.substantive_static_computations
            != self
                .cache_misses
                .checked_add(self.non_cacheable_requests)
                .ok_or("substantive computation equation overflow")?
        {
            return Err(
                "substantive computations must equal misses plus non-cacheable requests"
                    .to_string(),
            );
        }
        if self.cache_misses != self.cache_entries_constructed {
            return Err("cache misses must equal cache entries constructed".to_string());
        }
        if self.runtime_requests != self.runtime_executions {
            return Err("runtime requests must equal runtime executions".to_string());
        }
        if self.runtime_result_cache_hits != 0 {
            return Err("runtime-result cache hits must remain zero".to_string());
        }
        Ok(())
    }
}

pub(super) struct ValidationSession {
    policy: ValidationPolicy,
    identity: SessionIdentity,
    snapshots: Vec<SourceSnapshot>,
    initial_reference_load: Option<LoadedProgram>,
    optimized_load: Option<LoadedProgram>,
    observation_cache: BTreeMap<ObservationCacheKey, CommandObservation>,
    metrics: SessionMetrics,
}

impl ValidationSession {
    pub(super) fn from_paths(
        policy: ValidationPolicy,
        ordered_paths: Vec<PathBuf>,
    ) -> Result<Self, String> {
        let read_start = Instant::now();
        let sources = read_program_sources(&ordered_paths)?;
        let read_elapsed = read_start.elapsed();
        Self::from_loaded_sources(policy, sources, read_elapsed)
    }

    pub(super) fn from_snapshots(
        policy: ValidationPolicy,
        snapshots: Vec<SourceSnapshot>,
    ) -> Result<Self, String> {
        let sources = snapshots
            .iter()
            .map(|snapshot| LoadedSource {
                path: snapshot.path.clone(),
                display_path: snapshot.path.display().to_string(),
                bytes: snapshot.bytes.clone(),
                read: Duration::ZERO,
            })
            .collect::<Vec<_>>();
        Self::from_loaded_sources(policy, sources, Duration::ZERO)
    }

    fn from_loaded_sources(
        policy: ValidationPolicy,
        sources: Vec<LoadedSource>,
        read_elapsed: Duration,
    ) -> Result<Self, String> {
        if sources.is_empty() {
            return Err("validation session requires at least one source".to_string());
        }
        let identity = SessionIdentity::from_sources(&sources);
        let snapshots = sources
            .iter()
            .map(|source| SourceSnapshot {
                path: source.path.clone(),
                bytes: source.bytes.clone(),
            })
            .collect::<Vec<_>>();
        let loaded = load_program_from_sources(&sources)?;
        invalidate_process_local_analysis_caches();
        let mut metrics = SessionMetrics {
            distinct_source_identities: sources.len() as u64,
            source_reads: sources.len() as u64,
            parses: sources.len() as u64,
            initial_checks: sources.len() as u64,
            ..SessionMetrics::default()
        };
        metrics.phases.source_read += read_elapsed;
        for timing in &loaded.timings {
            metrics.phases.parse += timing.parse;
            metrics.phases.initial_check += timing.check;
        }
        let (initial_reference_load, optimized_load) = match policy {
            ValidationPolicy::Reference => (Some(loaded), None),
            ValidationPolicy::Optimized => (None, Some(loaded)),
        };
        Ok(Self {
            policy,
            identity,
            snapshots,
            initial_reference_load,
            optimized_load,
            observation_cache: BTreeMap::new(),
            metrics,
        })
    }

    pub(super) fn policy(&self) -> ValidationPolicy {
        self.policy
    }

    pub(super) fn identity(&self) -> &SessionIdentity {
        &self.identity
    }

    pub(super) fn snapshots(&self) -> Vec<SourceSnapshot> {
        self.snapshots.clone()
    }

    pub(super) fn metrics(&self) -> &SessionMetrics {
        &self.metrics
    }

    pub(super) fn into_metrics(self) -> SessionMetrics {
        self.metrics
    }

    pub(super) fn observe(
        &mut self,
        request: &ObservationRequest,
    ) -> Result<CommandObservation, String> {
        let options = parse_cli(request.arguments.clone()).map_err(|error| {
            format!(
                "validation request {} could not parse: {error}",
                request.conclusion_id
            )
        })?;
        self.validate_request(request, &options)?;
        if options.command == "run" {
            self.metrics.runtime_requests += 1;
            let observation = self.observe_fresh(&options)?;
            self.metrics.runtime_executions += 1;
            return Ok(observation);
        }

        self.metrics.static_requests += 1;
        let cacheable = self.policy == ValidationPolicy::Optimized
            && request.cache_eligibility == CacheEligibility::EligibleStatic;
        if !cacheable {
            self.metrics.non_cacheable_requests += 1;
            let observation = self.observe_fresh(&options)?;
            self.metrics.substantive_static_computations += 1;
            return Ok(observation);
        }

        self.metrics.cacheable_requests += 1;
        self.metrics.cache_lookups += 1;
        let key = ObservationCacheKey {
            session: self.identity.clone(),
            arguments: request.arguments.clone(),
        };
        let lookup_start = Instant::now();
        if let Some(observation) = self.observation_cache.get(&key) {
            self.metrics.phases.cache_lookup += lookup_start.elapsed();
            let hit_start = Instant::now();
            let observation = observation.clone();
            let hit_elapsed = hit_start.elapsed();
            self.metrics.phases.cache_hit += hit_elapsed;
            self.metrics.phases.cache_reuse += hit_elapsed;
            self.metrics.cache_hits += 1;
            self.metrics.cache_reuses += 1;
            return Ok(observation);
        }
        let lookup_elapsed = lookup_start.elapsed();
        self.metrics.phases.cache_lookup += lookup_elapsed;
        self.metrics.phases.cache_miss += lookup_elapsed;
        self.metrics.cache_misses += 1;
        let compute_start = Instant::now();
        let observation = self.observe_optimized_loaded(&options)?;
        self.metrics.phases.substantive_static_analysis += compute_start.elapsed();
        self.metrics.substantive_static_computations += 1;
        let construct_start = Instant::now();
        if self
            .observation_cache
            .insert(key, observation.clone())
            .is_some()
        {
            return Err("validation cache replaced an existing exact key".to_string());
        }
        self.metrics.phases.cache_entry_construction += construct_start.elapsed();
        self.metrics.cache_entries_constructed += 1;
        Ok(observation)
    }

    pub(super) fn assert_reference_predicate_cache_isolation(
        &mut self,
        request: &ObservationRequest,
    ) -> Result<CommandObservation, String> {
        if self.policy != ValidationPolicy::Reference {
            return Err("predicate cache isolation sabotage requires Reference policy".to_string());
        }
        let program = self
            .initial_reference_load
            .as_ref()
            .ok_or("predicate cache isolation sabotage requires the initial Reference load")?
            .program
            .clone();
        let stale = predicate::analyze_program(&program);
        let observation = self.observe(request)?;
        let retained = predicate::analyze_program(&program);
        if Arc::ptr_eq(&stale, &retained) {
            return Err(
                "Reference observation reused the deliberately stale predicate analysis"
                    .to_string(),
            );
        }
        Ok(observation)
    }

    fn validate_request(
        &self,
        request: &ObservationRequest,
        options: &CliOptions,
    ) -> Result<(), String> {
        if options.inputs != self.identity.ordered_paths {
            return Err(format!(
                "validation request {} input order does not equal its immutable session identity",
                request.conclusion_id
            ));
        }
        if options.show_timings {
            return Err(format!(
                "validation request {} cannot cache or compare nondeterministic --timings output",
                request.conclusion_id
            ));
        }
        if options.math_obligations_out_dir.is_some() {
            return Err(format!(
                "validation request {} cannot write external obligation files",
                request.conclusion_id
            ));
        }
        if request.cache_eligibility == CacheEligibility::EligibleStatic
            && !is_eligible_static_command(&options.command)
        {
            return Err(format!(
                "validation request {} marked non-static command `{}` cacheable",
                request.conclusion_id, options.command
            ));
        }
        if options.command == "run" && request.cache_eligibility != CacheEligibility::MandatoryFresh
        {
            return Err(format!(
                "validation request {} attempted to cache runtime state",
                request.conclusion_id
            ));
        }
        Ok(())
    }

    fn observe_fresh(&mut self, options: &CliOptions) -> Result<CommandObservation, String> {
        match self.policy {
            ValidationPolicy::Reference => {
                invalidate_process_local_analysis_caches();
                let loaded = if let Some(loaded) = self.initial_reference_load.take() {
                    loaded
                } else {
                    let read_start = Instant::now();
                    let sources = read_program_sources(&self.identity.ordered_paths)?;
                    self.metrics.phases.source_read += read_start.elapsed();
                    self.verify_sources(&sources)?;
                    self.metrics.source_reads += sources.len() as u64;
                    let loaded = load_program_from_sources(&sources)?;
                    self.metrics.parses += sources.len() as u64;
                    self.metrics.initial_checks += sources.len() as u64;
                    for timing in &loaded.timings {
                        self.metrics.phases.parse += timing.parse;
                        self.metrics.phases.initial_check += timing.check;
                    }
                    loaded
                };
                let analysis_start = Instant::now();
                let observation = observe_loaded_program(&loaded, options)?;
                if options.command == "run" {
                    self.metrics.phases.fresh_runtime_execution += analysis_start.elapsed();
                } else {
                    self.metrics.phases.substantive_static_analysis += analysis_start.elapsed();
                }
                Ok(observation)
            }
            ValidationPolicy::Optimized => {
                let analysis_start = Instant::now();
                let observation = self.observe_optimized_loaded(options)?;
                if options.command == "run" {
                    self.metrics.phases.fresh_runtime_execution += analysis_start.elapsed();
                } else {
                    self.metrics.phases.substantive_static_analysis += analysis_start.elapsed();
                }
                Ok(observation)
            }
        }
    }

    fn observe_optimized_loaded(&self, options: &CliOptions) -> Result<CommandObservation, String> {
        let loaded = self
            .optimized_load
            .as_ref()
            .ok_or("optimized validation session lost its immutable loaded program")?;
        observe_loaded_program(loaded, options)
    }

    fn verify_sources(&self, sources: &[LoadedSource]) -> Result<(), String> {
        let fresh = SessionIdentity::from_sources(sources);
        if fresh != self.identity {
            return Err(
                "validation source bytes, order, or semantic environment drifted".to_string(),
            );
        }
        Ok(())
    }
}

fn invalidate_process_local_analysis_caches() {
    // The product's process-local analysis accelerators intentionally use object
    // addresses. A Reference observation represents a fresh process/load and
    // must not inherit an address recycled by this long-lived test process.
    let sentinel = Program::default();
    callable::analyze_program(&sentinel);
    predicate::analyze_program(&sentinel);
    typed_failure::analyze_program(&sentinel);
}

fn is_eligible_static_command(command: &str) -> bool {
    matches!(
        command,
        "check"
            | "graph"
            | "resolve"
            | "type-env"
            | "type-check"
            | "core-preview"
            | "core-lower"
            | "core-verify"
            | "full-type-check"
            | "effect-check"
            | "ownership-check"
            | "resource-check"
            | "profile-check"
            | "ir-readiness"
    )
}

fn semantic_environment_identity() -> String {
    format!(
        "hum-validation-session-v1;os={};arch={};package={};canonical_seal_tier={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::env::var("HUM_CANONICAL_SEAL_EVIDENCE_TIER").unwrap_or_else(|_| "<unset>".to_string())
    )
}

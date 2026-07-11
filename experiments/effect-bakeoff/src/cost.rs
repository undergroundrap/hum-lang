#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ImplementationCost {
    pub nonblank_noncomment_lines: usize,
    pub dependency_count: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AnalysisCost {
    pub visited_corpus_nodes: usize,
    pub generated_facts: usize,
    pub generated_constraints: usize,
    pub normalization_steps: usize,
    pub maximum_live_items: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AnalysisTrace {
    cost: AnalysisCost,
    live_items: usize,
}

impl AnalysisTrace {
    pub fn visit_node(&mut self) {
        self.cost.visited_corpus_nodes += 1;
    }

    pub fn generate_fact(&mut self) {
        self.cost.generated_facts += 1;
    }

    pub fn generate_constraint(&mut self) {
        self.cost.generated_constraints += 1;
    }

    pub fn normalization_step(&mut self) {
        self.cost.normalization_steps += 1;
    }

    pub fn enter_live_item(&mut self) {
        self.live_items += 1;
        self.cost.maximum_live_items = self.cost.maximum_live_items.max(self.live_items);
    }

    pub fn leave_live_item(&mut self) {
        self.live_items = self.live_items.saturating_sub(1);
    }

    pub fn measured(&self) -> AnalysisCost {
        self.cost.clone()
    }

    pub fn check_reported(&self, reported: &AnalysisCost) -> bool {
        &self.cost == reported
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticCost {
    pub primary_diagnostic_count: usize,
    pub required_site_count: usize,
    pub covered_required_site_count: usize,
    pub rendered_utf8_bytes: usize,
    pub candidate_native_term_count: usize,
    pub has_model_neutral_repair: bool,
}

pub fn measure_implementation(source: &str, manifest: &str) -> ImplementationCost {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Block(usize),
        String,
        Char,
        Raw(usize),
    }
    let mut state = State::Normal;
    let mut nonblank_noncomment_lines = 0;
    for line in source.lines() {
        let bytes = line.as_bytes();
        let mut i = 0;
        let mut has_code = !matches!(state, State::Normal | State::Block(_));
        while i < bytes.len() {
            match state {
                State::Block(depth) => {
                    if i + 1 < bytes.len() && &bytes[i..i + 2] == b"/*" {
                        state = State::Block(depth + 1);
                        i += 2;
                    } else if i + 1 < bytes.len() && &bytes[i..i + 2] == b"*/" {
                        state = if depth == 1 {
                            State::Normal
                        } else {
                            State::Block(depth - 1)
                        };
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                State::String => {
                    has_code = true;
                    if bytes[i] == b'\\' {
                        i = (i + 2).min(bytes.len());
                    } else if bytes[i] == b'"' {
                        state = State::Normal;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                State::Char => {
                    has_code = true;
                    if bytes[i] == b'\\' {
                        i = (i + 2).min(bytes.len());
                    } else if bytes[i] == b'\'' {
                        state = State::Normal;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                State::Raw(hashes) => {
                    has_code = true;
                    if bytes[i] == b'"'
                        && i + 1 + hashes <= bytes.len()
                        && bytes[i + 1..i + 1 + hashes].iter().all(|b| *b == b'#')
                    {
                        state = State::Normal;
                        i += 1 + hashes;
                    } else {
                        i += 1;
                    }
                }
                State::Normal => {
                    if i + 1 < bytes.len() && &bytes[i..i + 2] == b"//" {
                        break;
                    }
                    if i + 1 < bytes.len() && &bytes[i..i + 2] == b"/*" {
                        state = State::Block(1);
                        i += 2;
                        continue;
                    }
                    let raw_start = if bytes[i] == b'r' {
                        Some(i + 1)
                    } else if i + 1 < bytes.len() && bytes[i] == b'b' && bytes[i + 1] == b'r' {
                        Some(i + 2)
                    } else {
                        None
                    };
                    if let Some(mut cursor) = raw_start {
                        while cursor < bytes.len() && bytes[cursor] == b'#' {
                            cursor += 1;
                        }
                        if cursor < bytes.len() && bytes[cursor] == b'"' {
                            has_code = true;
                            state = State::Raw(cursor - raw_start.unwrap());
                            i = cursor + 1;
                            continue;
                        }
                    }
                    if bytes[i] == b'"' {
                        has_code = true;
                        state = State::String;
                        i += 1;
                        continue;
                    }
                    let char_literal = bytes[i] == b'\''
                        && ((i + 2 < bytes.len() && bytes[i + 2] == b'\'')
                            || (i + 3 < bytes.len()
                                && bytes[i + 1] == b'\\'
                                && bytes[i + 3] == b'\''));
                    if char_literal {
                        has_code = true;
                        state = State::Char;
                        i += 1;
                        continue;
                    }
                    if !bytes[i].is_ascii_whitespace() {
                        has_code = true;
                    }
                    i += 1;
                }
            }
        }
        if has_code {
            nonblank_noncomment_lines += 1;
        }
    }
    let mut in_dependencies = false;
    let mut dependency_count = 0;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            let section = line.trim_matches(&['[', ']'][..]);
            in_dependencies = section == "dependencies"
                || section == "dev-dependencies"
                || section == "build-dependencies"
                || section.ends_with(".dependencies")
                || section.ends_with(".dev-dependencies")
                || section.ends_with(".build-dependencies");
        } else if in_dependencies
            && !line.is_empty()
            && !line.starts_with('#')
            && line.contains('=')
        {
            dependency_count += 1;
        }
    }
    ImplementationCost {
        nonblank_noncomment_lines,
        dependency_count,
    }
}

pub fn measure_inventory(sources: &[&str], manifests: &[&str]) -> ImplementationCost {
    let lines = sources
        .iter()
        .map(|source| measure_implementation(source, "").nonblank_noncomment_lines)
        .sum();
    let dependencies = manifests
        .iter()
        .map(|manifest| measure_implementation("", manifest).dependency_count)
        .sum();
    ImplementationCost {
        nonblank_noncomment_lines: lines,
        dependency_count: dependencies,
    }
}

pub fn measure_diagnostic(
    rendered: &str,
    required_sites: &[String],
    covered_sites: &[String],
    candidate_native_terms: &[String],
    repair: &str,
) -> DiagnosticCost {
    DiagnosticCost {
        primary_diagnostic_count: usize::from(!rendered.is_empty()),
        required_site_count: required_sites.len(),
        covered_required_site_count: required_sites
            .iter()
            .filter(|site| covered_sites.contains(site))
            .count(),
        rendered_utf8_bytes: rendered.len(),
        candidate_native_term_count: candidate_native_terms
            .iter()
            .filter(|term| rendered.contains(term.as_str()))
            .count(),
        has_model_neutral_repair: !repair.trim().is_empty(),
    }
}

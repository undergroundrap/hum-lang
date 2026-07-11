use crate::cost::{ImplementationCost, measure_inventory};

pub struct CheckedInInventory {
    sources: &'static [&'static str],
    manifests: &'static [&'static str],
}

impl CheckedInInventory {
    pub fn measure(&self) -> ImplementationCost {
        measure_inventory(self.sources, self.manifests)
    }
}

pub fn candidate_inventory(candidate_id: &str) -> Option<CheckedInInventory> {
    match candidate_id {
        #[cfg(test)]
        "mock_candidate" => Some(CheckedInInventory {
            sources: &[include_str!("lib.rs")],
            manifests: &[include_str!("../Cargo.toml")],
        }),
        _ => None,
    }
}

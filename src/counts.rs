use std::collections::HashMap;

pub(crate) type Count = u64;

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct MixedKey {
    pub(crate) image: Option<String>,
    pub(crate) raw: String,
}

#[derive(Default)]
pub(crate) struct SingleCounts {
    pub(crate) leaf: HashMap<String, Count>,
    pub(crate) all: HashMap<String, Count>,
    pub(crate) addr: HashMap<String, Count>,
}

#[derive(Default)]
pub(crate) struct MixedCounts {
    pub(crate) leaf: HashMap<MixedKey, Count>,
    pub(crate) all: HashMap<MixedKey, Count>,
}

pub(crate) fn add_count<K>(counts: &mut HashMap<K, Count>, key: K, weight: Count)
where
    K: Eq + std::hash::Hash,
{
    *counts.entry(key).or_default() += weight;
}

pub(crate) fn sorted_top<K>(counts: &HashMap<K, Count>, limit: usize) -> Vec<(K, Count)>
where
    K: Clone + Eq + Ord + std::hash::Hash,
{
    let mut entries: Vec<_> = counts
        .iter()
        .map(|(key, count)| (key.clone(), *count))
        .collect();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries.truncate(limit);
    entries
}

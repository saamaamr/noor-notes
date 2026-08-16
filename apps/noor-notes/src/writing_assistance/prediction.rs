use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionModel {
    bigrams: BTreeMap<String, BTreeMap<String, CandidateCount>>,
    trigrams: BTreeMap<String, BTreeMap<String, CandidateCount>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CandidateCount {
    display: String,
    count: u64,
}

#[derive(Clone, Debug)]
struct RankedCandidate {
    normalized: String,
    display: String,
    trigram_count: u64,
    bigram_count: u64,
}

impl PredictionModel {
    pub fn train(&mut self, documents: &[String]) {
        for document in documents {
            let words = UnicodeSegmentation::unicode_words(document.as_str())
                .map(|word| (normalize(word), word.to_owned()))
                .filter(|(normalized, _)| !normalized.is_empty())
                .collect::<Vec<_>>();

            for pair in words.windows(2) {
                increment(&mut self.bigrams, &pair[0].0, &pair[1].0, &pair[1].1);
            }
            for triple in words.windows(3) {
                let context = format!("{}\u{0}{}", triple[0].0, triple[1].0);
                increment(&mut self.trigrams, &context, &triple[2].0, &triple[2].1);
            }
        }
    }

    pub fn suggest(&self, context: &str, partial: &str, limit: usize) -> Vec<String> {
        let words = UnicodeSegmentation::unicode_words(context)
            .map(normalize)
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        let Some(last) = words.last() else {
            return Vec::new();
        };
        let partial = normalize(partial);
        let trigram_key =
            (words.len() >= 2).then(|| format!("{}\u{0}{}", words[words.len() - 2], last));
        let trigram = trigram_key.as_ref().and_then(|key| self.trigrams.get(key));
        let bigram = self.bigrams.get(last);
        let candidates = trigram
            .into_iter()
            .flat_map(|values| values.keys())
            .chain(bigram.into_iter().flat_map(|values| values.keys()))
            .cloned()
            .collect::<BTreeSet<_>>();

        let mut ranked = candidates
            .into_iter()
            .filter(|candidate| candidate.starts_with(&partial))
            .map(|normalized| {
                let trigram_value = trigram.and_then(|values| values.get(&normalized));
                let bigram_value = bigram.and_then(|values| values.get(&normalized));
                RankedCandidate {
                    display: trigram_value
                        .or(bigram_value)
                        .map(|value| value.display.clone())
                        .unwrap_or_else(|| normalized.clone()),
                    trigram_count: trigram_value.map_or(0, |value| value.count),
                    bigram_count: bigram_value.map_or(0, |value| value.count),
                    normalized,
                }
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .trigram_count
                .cmp(&left.trigram_count)
                .then_with(|| right.bigram_count.cmp(&left.bigram_count))
                .then_with(|| left.normalized.cmp(&right.normalized))
        });
        ranked
            .into_iter()
            .take(limit.min(3))
            .map(|candidate| candidate.display)
            .collect()
    }

    pub fn prune(&mut self, maximum_entries: usize) {
        let remove_count = self.entry_count().saturating_sub(maximum_entries);
        let mut candidates = pruning_candidates(&self.bigrams, "b");
        candidates.extend(pruning_candidates(&self.trigrams, "t"));
        candidates.sort_by(|left, right| {
            left.3
                .cmp(&right.3)
                .then_with(|| left.0.cmp(&right.0))
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        for (kind, context, candidate, _) in candidates.into_iter().take(remove_count) {
            let table = if kind == "b" {
                &mut self.bigrams
            } else {
                &mut self.trigrams
            };
            if let Some(values) = table.get_mut(&context) {
                values.remove(&candidate);
                if values.is_empty() {
                    table.remove(&context);
                }
            }
        }
    }

    pub fn entry_count(&self) -> usize {
        count_entries(&self.bigrams) + count_entries(&self.trigrams)
    }
}

fn normalize(value: &str) -> String {
    value.to_lowercase()
}

fn increment(
    table: &mut BTreeMap<String, BTreeMap<String, CandidateCount>>,
    context: &str,
    candidate: &str,
    display: &str,
) {
    let value = table
        .entry(context.to_owned())
        .or_default()
        .entry(candidate.to_owned())
        .or_insert_with(|| CandidateCount {
            display: display.to_owned(),
            count: 0,
        });
    value.count = value.count.saturating_add(1);
}

fn count_entries(table: &BTreeMap<String, BTreeMap<String, CandidateCount>>) -> usize {
    table.values().map(BTreeMap::len).sum()
}

fn pruning_candidates(
    table: &BTreeMap<String, BTreeMap<String, CandidateCount>>,
    kind: &'static str,
) -> Vec<(&'static str, String, String, u64)> {
    table
        .iter()
        .flat_map(|(context, values)| {
            values.iter().map(move |(candidate, value)| {
                (kind, context.clone(), candidate.clone(), value.count)
            })
        })
        .collect()
}

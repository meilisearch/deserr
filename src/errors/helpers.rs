//! Provide generic function that could be useful when creating your own
//! error type.

use strsim::damerau_levenshtein;

/// Compute a did you mean message from a received string and a list of
/// accepted strings.
pub fn did_you_mean(received: &str, accepted: &[&str]) -> String {
    let typo_allowed = match received.len() {
        // no typos are allowed, we can early return
        0..=3 => return String::new(),
        4..=7 => 1,
        8..=12 => 2,
        13..=17 => 3,
        18..=24 => 4,
        _ => 5,
    };
    match accepted
        .iter()
        .map(|accepted| (accepted, damerau_levenshtein(received, accepted)))
        .filter(|(_, distance)| distance <= &typo_allowed)
        .min_by(|(_, d1), (_, d2)| d1.cmp(d2))
    {
        None => String::new(),
        Some((accepted, _)) => format!("did you mean `{}`? ", accepted),
    }
}

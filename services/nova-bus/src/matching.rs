//! Shared topic-matching logic used by both the broker (fan-out on
//! `Publish`) and the client (demuxing inbound `Publish` to the right local
//! subscription channel). Lives in the client/proto crate — not the broker
//! crate — precisely so the client never has to depend on broker internals
//! to reuse it (docs/02-REPOSITORY-STRUCTURE.md §3 rule 2).

/// Per docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md §7: an exact match, or a
/// trailing-`.*` prefix match.
pub fn topic_matches(pattern: &str, topic: &str) -> bool {
    match pattern.strip_suffix(".*") {
        Some(prefix) => topic.starts_with(prefix) && topic.len() > prefix.len(),
        None => pattern == topic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_and_wildcard_matching() {
        assert!(topic_matches("nova.theme.changed", "nova.theme.changed"));
        assert!(!topic_matches("nova.theme.changed", "nova.theme.other"));
        assert!(topic_matches("nova.package.*", "nova.package.install_complete"));
        assert!(!topic_matches("nova.package.*", "nova.session.launch"));
        assert!(!topic_matches("nova.package.*", "nova.package"));
    }
}

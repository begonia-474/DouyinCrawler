//! SelectionTracker — single source of truth for aweme_id selection state.
//!
//! Tracks requested, seen, and planned IDs across paged pagination and
//! classifies terminal missing/unavailable outcomes.

use std::collections::HashSet;

/// Selection state for a paged download with optional explicit aweme_ids.
///
/// When `requested` is empty the tracker is inactive (full download).
#[derive(Debug, Clone)]
pub struct SelectionTracker {
    /// Normalized, deduplicated, insertion-order-preserved requested IDs.
    requested: Vec<String>,
    /// Membership index for requested IDs; kept alongside the ordered vector.
    requested_set: HashSet<String>,
    /// Set of requested IDs that have appeared in at least one `page_aweme_ids`.
    seen: HashSet<String>,
    /// Set of requested IDs that have produced at least one media item.
    planned: HashSet<String>,
}

impl SelectionTracker {
    /// Build a tracker from the raw user-supplied `aweme_ids`.
    ///
    /// Normalization trims whitespace, drops empty strings, and deduplicates
    /// while preserving first-occurrence order.
    pub fn new(raw: Vec<String>) -> Self {
        let requested = Self::normalize(raw);
        let requested_set = requested.iter().cloned().collect();
        Self {
            requested,
            requested_set,
            seen: HashSet::new(),
            planned: HashSet::new(),
        }
    }

    /// Whether a selection is active (non-empty requested set).
    pub fn is_active(&self) -> bool {
        !self.requested.is_empty()
    }

    /// The stabilized ordered list of requested IDs.
    #[cfg(test)]
    pub fn requested(&self) -> &[String] {
        &self.requested
    }

    /// Mark that `page_aweme_ids` appeared in the current page.
    ///
    /// Only requested IDs are tracked; non-requested IDs are ignored.
    pub fn mark_seen(&mut self, page_aweme_ids: &[String]) {
        for id in page_aweme_ids {
            if self.requested_set.contains(id) {
                self.seen.insert(id.clone());
            }
        }
    }

    /// Mark that an aweme_id has produced at least one media item.
    #[cfg(test)]
    pub fn mark_planned(&mut self, aweme_id: &str) {
        if self.requested_set.contains(aweme_id) {
            self.planned.insert(aweme_id.to_string());
        }
    }

    /// Keep complete media groups for requested works that have not already
    /// produced a plan on an earlier page, then mark those works as planned.
    pub fn take_unplanned_items<T>(
        &mut self,
        items: Vec<T>,
        get_aweme_id: &impl Fn(&T) -> &str,
    ) -> Vec<T> {
        if !self.is_active() {
            return items;
        }
        let page_items: Vec<T> = items
            .into_iter()
            .filter(|item| {
                let aweme_id = get_aweme_id(item);
                self.requested_set.contains(aweme_id) && !self.planned.contains(aweme_id)
            })
            .collect();
        let planned_ids: HashSet<String> = page_items
            .iter()
            .map(|item| get_aweme_id(item).to_string())
            .collect();
        self.planned.extend(planned_ids);
        page_items
    }

    /// All requested IDs have been seen at least once across pages.
    pub fn all_seen(&self) -> bool {
        self.requested.iter().all(|id| self.seen.contains(id))
    }

    /// All seen IDs have been planned (produced at least one media item).
    #[cfg(test)]
    pub fn all_planned(&self) -> bool {
        self.requested.iter().all(|id| self.planned.contains(id))
    }

    /// IDs that were never seen in any page (stable, in request order).
    pub fn missing_ids(&self) -> Vec<&str> {
        self.requested
            .iter()
            .filter(|id| !self.seen.contains(id.as_str()))
            .map(|s| s.as_str())
            .collect()
    }

    /// IDs that were seen but never produced any media item (stable, in request order).
    pub fn unavailable_ids(&self) -> Vec<&str> {
        self.requested
            .iter()
            .filter(|id| self.seen.contains(id.as_str()) && !self.planned.contains(id.as_str()))
            .map(|s| s.as_str())
            .collect()
    }

    /// Build a formatted selection-summary error string.
    ///
    /// Returns `None` when there are no missing or unavailable IDs.
    pub fn summary_error(&self) -> Option<String> {
        let missing = self.missing_ids();
        let unavailable = self.unavailable_ids();
        if missing.is_empty() && unavailable.is_empty() {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        if !missing.is_empty() {
            parts.push(format!("missing_aweme_ids=[{}]", missing.join(",")));
        }
        if !unavailable.is_empty() {
            parts.push(format!("unavailable_aweme_ids=[{}]", unavailable.join(",")));
        }
        Some(parts.join("; "))
    }

    /// Append stable ordered selection progress to a page/protocol error.
    pub fn contextualize(&self, message: String) -> String {
        if !self.is_active() {
            return message;
        }
        let seen = self.ordered_members(&self.seen);
        let planned = self.ordered_members(&self.planned);
        format!(
            "{message}; selection_state requested=[{}]; seen=[{}]; planned=[{}]",
            self.requested.join(","),
            seen.join(","),
            planned.join(",")
        )
    }

    fn ordered_members<'a>(&'a self, members: &HashSet<String>) -> Vec<&'a str> {
        self.requested
            .iter()
            .filter(|id| members.contains(id.as_str()))
            .map(String::as_str)
            .collect()
    }

    fn normalize(raw: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for id in raw {
            let trimmed = id.trim().to_string();
            if trimmed.is_empty() || seen.contains(trimmed.as_str()) {
                continue;
            }
            seen.insert(trimmed.clone());
            result.push(trimmed);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_inactive() {
        let t = SelectionTracker::new(vec![]);
        assert!(!t.is_active());
        assert!(t.requested().is_empty());
    }

    #[test]
    fn normalizes_trims_and_deduplicates() {
        let t = SelectionTracker::new(vec![
            "  a ".to_string(),
            "b".to_string(),
            "".to_string(),
            "  ".to_string(),
            "a".to_string(),
            "b".to_string(),
        ]);
        assert!(t.is_active());
        assert_eq!(t.requested(), &["a", "b"]);
    }

    #[test]
    fn preserves_input_order() {
        let t = SelectionTracker::new(vec![
            "z".to_string(),
            "a".to_string(),
            "m".to_string(),
            "z".to_string(),
        ]);
        assert_eq!(t.requested(), &["z", "a", "m"]);
    }

    #[test]
    fn seen_tracks_only_requested_ids() {
        let mut t = SelectionTracker::new(vec!["target".to_string()]);
        t.mark_seen(&["target".to_string(), "other".to_string()]);
        assert!(t.all_seen());
        assert!(t.missing_ids().is_empty());
    }

    #[test]
    fn duplicate_seen_is_idempotent() {
        let mut t = SelectionTracker::new(vec!["a".to_string(), "b".to_string()]);
        t.mark_seen(&["a".to_string()]);
        t.mark_seen(&["a".to_string()]);
        assert_eq!(t.seen.len(), 1);
        assert!(!t.all_seen());
        t.mark_seen(&["b".to_string()]);
        assert!(t.all_seen());
    }

    #[test]
    fn missing_classified_correctly() {
        let mut t = SelectionTracker::new(vec!["seen-one".to_string(), "never-seen".to_string()]);
        t.mark_seen(&["seen-one".to_string()]);
        t.mark_planned("seen-one");
        assert_eq!(t.missing_ids(), vec!["never-seen"]);
        assert!(t.unavailable_ids().is_empty());
    }

    #[test]
    fn unavailable_classified_correctly() {
        let mut t = SelectionTracker::new(vec!["has-media".to_string(), "no-media".to_string()]);
        t.mark_seen(&["has-media".to_string(), "no-media".to_string()]);
        t.mark_planned("has-media");
        assert!(t.missing_ids().is_empty());
        assert_eq!(t.unavailable_ids(), vec!["no-media"]);
    }

    #[test]
    fn all_hit_produces_no_errors() {
        let mut t = SelectionTracker::new(vec!["a".to_string(), "b".to_string()]);
        t.mark_seen(&["a".to_string(), "b".to_string()]);
        t.mark_planned("a");
        t.mark_planned("b");
        assert!(t.all_seen());
        assert!(t.all_planned());
        assert!(t.missing_ids().is_empty());
        assert!(t.unavailable_ids().is_empty());
        assert!(t.summary_error().is_none());
    }

    #[test]
    fn take_unplanned_items_keeps_complete_group_once() {
        let mut t = SelectionTracker::new(vec!["work".to_string(), "later".to_string()]);
        let first = t.take_unplanned_items(
            vec![
                ("work", "media-1"),
                ("work", "media-2"),
                ("other", "ignored"),
            ],
            &|item| item.0,
        );
        assert_eq!(first, vec![("work", "media-1"), ("work", "media-2")]);
        let repeated = t
            .take_unplanned_items(vec![("work", "media-1"), ("later", "media-1")], &|item| {
                item.0
            });
        assert_eq!(repeated, vec![("later", "media-1")]);
    }

    #[test]
    fn contextualize_preserves_requested_order() {
        let mut t = SelectionTracker::new(vec!["second".to_string(), "first".to_string()]);
        t.mark_seen(&["first".to_string()]);
        t.mark_planned("first");
        assert_eq!(
            t.contextualize("page failed".to_string()),
            "page failed; selection_state requested=[second,first]; seen=[first]; planned=[first]"
        );
    }

    #[test]
    fn summary_error_joins_missing_and_unavailable() {
        let mut t = SelectionTracker::new(vec![
            "missing-one".to_string(),
            "missing-two".to_string(),
            "unavail".to_string(),
            "ok".to_string(),
        ]);
        t.mark_seen(&["unavail".to_string(), "ok".to_string()]);
        t.mark_planned("ok");
        let s = t.summary_error().unwrap();
        assert!(
            s.contains("missing_aweme_ids=[missing-one,missing-two]"),
            "{s}"
        );
        assert!(s.contains("unavailable_aweme_ids=[unavail]"), "{s}");
    }
}

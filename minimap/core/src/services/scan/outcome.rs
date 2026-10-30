#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanOutcome {
    // the chunk was probed; needs_retry when a placeholder was met
    Scanned { needs_retry: bool },
    NotLoaded,
    TopPlaceholder,
    // the host reached its loaded-chunk limit before this candidate
    NotProbed,
}

use crate::structs::waypoint::Waypoint;

// the waypoint list's search: a case-insensitive part of the name, in one dimension
pub struct WaypointFilter {
    // the query as typed, so an unchanged query is not lowered again
    raw: String,
    query: Vec<char>,
}

impl WaypointFilter {
    // the longest search the list's field takes, in characters
    pub const QUERY_CHARS: usize = 32;

    pub fn new() -> Self {
        Self { raw: String::new(), query: Vec::new() }
    }

    pub fn set_query(&mut self, text: &str) {
        if self.raw == text {
            return;
        }
        self.raw.clear();
        self.raw.push_str(text);
        self.query.clear();
        self.query.extend(text.chars().flat_map(char::to_lowercase));
    }

    // compares lowered characters as it walks the name, so nothing is allocated
    pub fn matches(&self, name: &str) -> bool {
        if self.query.is_empty() {
            return true;
        }
        name.char_indices().any(|(start, _)| {
            let mut lowered = name[start..].chars().flat_map(char::to_lowercase);
            self.query.iter().all(|q| lowered.next() == Some(*q))
        })
    }

    // indices into `waypoints`, in store order; the session's "hide waypoints" does not apply
    pub fn collect(&self, waypoints: &[Waypoint], dim: i32, out: &mut Vec<usize>) {
        out.clear();
        out.extend(waypoints.iter().enumerate().filter(|(_, w)| w.dim == dim && self.matches(&w.name)).map(|(i, _)| i));
    }

    // waypoints in one dimension, whatever the query
    pub fn count_in(waypoints: &[Waypoint], dim: i32) -> usize {
        waypoints.iter().filter(|w| w.dim == dim).count()
    }
}

impl Default for WaypointFilter {
    fn default() -> Self {
        Self::new()
    }
}

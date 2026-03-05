use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::db::Session;

/// Which fields to match against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchField {
    All,
    Title,
    Message,
    Directory,
}

/// Match indices for highlighted fields.
#[derive(Default)]
pub struct MatchIndices {
    pub title: Vec<usize>,
    pub last_input: Vec<usize>,
    pub directory: Vec<usize>,
}

/// A session paired with its fuzzy match score and highlight indices.
pub struct ScoredSession {
    pub session: Session,
    pub score: i64,
    pub indices: MatchIndices,
}

/// Parse a query string for a field prefix (`title:`, `mes:`, `dir:`).
/// Returns the match field and the remaining query text.
pub fn parse_query(raw: &str) -> (MatchField, &str) {
    if let Some(rest) = raw.strip_prefix("title:") {
        (MatchField::Title, rest)
    } else if let Some(rest) = raw.strip_prefix("mes:") {
        (MatchField::Message, rest)
    } else if let Some(rest) = raw.strip_prefix("dir:") {
        (MatchField::Directory, rest)
    } else {
        (MatchField::All, raw)
    }
}

/// Filter and score sessions by fuzzy matching.
/// When `sort_by_date` is true, matched results keep their date-descending order
/// rather than being sorted by score.
pub fn filter_sessions(
    sessions: &[Session],
    query: &str,
    sort_by_date: bool,
) -> Vec<ScoredSession> {
    let (field, pattern) = parse_query(query);

    if pattern.is_empty() {
        return sessions
            .iter()
            .enumerate()
            .map(|(i, s)| ScoredSession {
                session: s.clone(),
                score: -(i as i64), // preserve date-desc order from DB
                indices: MatchIndices::default(),
            })
            .collect();
    }

    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<ScoredSession> = sessions
        .iter()
        .enumerate()
        .filter_map(|(i, session)| {
            let mut indices = MatchIndices::default();

            let score = match field {
                MatchField::Title => match matcher.fuzzy_indices(&session.title, pattern) {
                    Some((s, idx)) => {
                        indices.title = idx;
                        s
                    }
                    None => 0,
                },
                MatchField::Message => match matcher.fuzzy_indices(&session.last_input, pattern) {
                    Some((s, idx)) => {
                        indices.last_input = idx;
                        s
                    }
                    None => 0,
                },
                MatchField::Directory => match matcher.fuzzy_indices(&session.directory, pattern) {
                    Some((s, idx)) => {
                        indices.directory = idx;
                        s
                    }
                    None => 0,
                },
                MatchField::All => {
                    let t = matcher.fuzzy_indices(&session.title, pattern);
                    let d = matcher.fuzzy_indices(&session.directory, pattern);
                    let m = matcher.fuzzy_indices(&session.last_input, pattern);

                    let t_score = t.as_ref().map_or(0, |(s, _)| *s);
                    let d_score = d.as_ref().map_or(0, |(s, _)| *s);
                    let m_score = m.as_ref().map_or(0, |(s, _)| *s);

                    // Store indices for all matching fields
                    if let Some((_, idx)) = t {
                        indices.title = idx;
                    }
                    if let Some((_, idx)) = d {
                        indices.directory = idx;
                    }
                    if let Some((_, idx)) = m {
                        indices.last_input = idx;
                    }

                    t_score.max(d_score).max(m_score)
                }
            };

            if score > 0 {
                Some(ScoredSession {
                    session: session.clone(),
                    score: if sort_by_date { -(i as i64) } else { score },
                    indices,
                })
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.score.cmp(&a.score));
    scored
}

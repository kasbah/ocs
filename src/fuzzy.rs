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

/// A session paired with its fuzzy match score.
pub struct ScoredSession {
    pub session: Session,
    pub score: i64,
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
            })
            .collect();
    }

    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<ScoredSession> = sessions
        .iter()
        .enumerate()
        .filter_map(|(i, session)| {
            let score = match field {
                MatchField::Title => matcher.fuzzy_match(&session.title, pattern).unwrap_or(0),
                MatchField::Message => matcher
                    .fuzzy_match(&session.last_input, pattern)
                    .unwrap_or(0),
                MatchField::Directory => matcher
                    .fuzzy_match(&session.directory, pattern)
                    .unwrap_or(0),
                MatchField::All => {
                    let t = matcher.fuzzy_match(&session.title, pattern).unwrap_or(0);
                    let d = matcher
                        .fuzzy_match(&session.directory, pattern)
                        .unwrap_or(0);
                    let m = matcher
                        .fuzzy_match(&session.last_input, pattern)
                        .unwrap_or(0);
                    t.max(d).max(m)
                }
            };

            if score > 0 {
                Some(ScoredSession {
                    session: session.clone(),
                    // When sorting by date, use negative index to preserve DB order
                    score: if sort_by_date { -(i as i64) } else { score },
                })
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.score.cmp(&a.score));
    scored
}

use serde::Deserialize;
use std::path::Path;

/// Shortest query that also matches its plural.
const MIN_PLURAL_LEN: usize = 3;
/// Shortest query that may match the start of a longer word ("appl" → apple).
/// Three letters is too loose: "nak" is Malay for "want", and must not reach
/// "naked".
const MIN_PREFIX_LEN: usize = 4;
/// Shortest query that may match inside a word ("berry" → strawberry).
const MIN_INSIDE_LEN: usize = 5;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub file: String,
    /// Name and tags as `padded()` text, built once at load.
    name_text: String,
    tag_texts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SymbolIndex {
    symbols: Vec<Symbol>,
}

#[derive(Deserialize)]
struct SymbolEntry {
    name: String,
    file: String,
    tags: Vec<String>,
}

/// Lowercase words separated by single spaces: "Zero-variant_1" → "zero variant 1".
fn normalize(text: &str) -> String {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalized text with a space at each end, so " cat " finds whole words.
fn padded(text: &str) -> String {
    format!(" {} ", normalize(text))
}

/// How well `query` (normalized) fits one padded text.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Fit {
    None,
    Inside,
    Prefix,
    Words,
    WordsAtStart,
    Exact,
}

fn fit(text: &str, query: &str) -> Fit {
    let len = query.chars().count();
    let words = format!(" {query} ");
    // "dog" also finds "dogs", but "hi" must not find "his".
    let plural = if len >= MIN_PLURAL_LEN {
        format!(" {query}s ")
    } else {
        words.clone()
    };

    if text == words {
        Fit::Exact
    } else if text.starts_with(&words) || text.starts_with(&plural) {
        Fit::WordsAtStart
    } else if text.contains(&words) || text.contains(&plural) {
        Fit::Words
    } else if len >= MIN_PREFIX_LEN && text.contains(&format!(" {query}")) {
        Fit::Prefix
    } else if len >= MIN_INSIDE_LEN && text.contains(query) {
        Fit::Inside
    } else {
        Fit::None
    }
}

impl Symbol {
    fn new(name: String, file: String, tags: &[String]) -> Self {
        Symbol {
            name_text: padded(&name),
            tag_texts: tags.iter().map(|tag| padded(tag)).collect(),
            name,
            file,
        }
    }

    fn best_tag_fit(&self, query: &str) -> Fit {
        self.tag_texts
            .iter()
            .map(|tag| fit(tag, query))
            .fold(Fit::None, |best, f| if f > best { f } else { best })
    }

    fn score(&self, query: &str, terms: &[&str]) -> i32 {
        // Whole words beat word starts, and a name beats a tag at the same
        // level: "want" finds the tag "want" before the name "wanted".
        match (fit(&self.name_text, query), self.best_tag_fit(query)) {
            (Fit::Exact, _) => return 100,
            (Fit::WordsAtStart, _) => return 80,
            (Fit::Words, _) => return 70,
            (_, Fit::Exact) => return 60,
            (_, Fit::WordsAtStart | Fit::Words) => return 50,
            (Fit::Prefix, _) => return 40,
            (_, Fit::Prefix) => return 35,
            (Fit::Inside, _) => return 30,
            (_, Fit::Inside) => return 25,
            _ => {}
        }

        // Several words: count the ones found anywhere in the name or tags.
        let matched_terms = terms
            .iter()
            .filter(|term| {
                fit(&self.name_text, term) != Fit::None || self.best_tag_fit(term) != Fit::None
            })
            .count();

        10 * matched_terms as i32
    }
}

impl SymbolIndex {
    pub fn load(media_dir: &str) -> Self {
        let mut symbols = Vec::new();

        let media_path = Path::new(media_dir);
        let entries = std::fs::read_dir(media_path).expect("failed to read media directory");

        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }

            let library = entry.file_name().to_string_lossy().to_string();
            let yaml_path = media_path.join(format!("{}.yaml", library));

            if !yaml_path.exists() {
                tracing::warn!("no {}.yaml found in {}", library, media_dir);
                continue;
            }

            let content = std::fs::read_to_string(&yaml_path)
                .unwrap_or_else(|_| panic!("failed to read {}.yaml", library));
            let entries: Vec<SymbolEntry> = serde_yaml_ng::from_str(&content)
                .unwrap_or_else(|_| panic!("failed to parse {}.yaml", library));

            for entry in entries {
                let file = format!("media/{}/{}", library, entry.file);
                symbols.push(Symbol::new(entry.name, file, &entry.tags));
            }
        }

        tracing::info!("loaded {} symbols from {}", symbols.len(), media_dir);
        SymbolIndex { symbols }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<&Symbol> {
        let query = normalize(query);
        let results = self.search_normalized(&query, limit);

        // Nothing for "dogs": try "dog".
        match query.strip_suffix('s') {
            Some(singular) if results.is_empty() && singular.chars().count() >= MIN_PLURAL_LEN => {
                self.search_normalized(singular, limit)
            }
            _ => results,
        }
    }

    fn search_normalized(&self, query: &str, limit: usize) -> Vec<&Symbol> {
        let terms: Vec<&str> = query.split(' ').filter(|t| !t.is_empty()).collect();
        if terms.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(&Symbol, i32)> = self
            .symbols
            .iter()
            .filter_map(|symbol| {
                let score = symbol.score(query, &terms);
                if score > 0 {
                    Some((symbol, score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(limit).map(|(s, _)| s).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Case {
        q: String,
        lang: String,
        expect: Vec<String>,
        #[serde(default)]
        avoid: Vec<String>,
    }

    /// Malay searches that must keep passing. Raise it as search improves.
    const MS_FLOOR: usize = 1;

    fn found(case: &Case, results: &[&Symbol]) -> bool {
        results.iter().take(5).any(|symbol| {
            case.expect
                .iter()
                .any(|word| symbol.name_text.contains(&format!(" {word} ")))
        })
    }

    fn unwanted(case: &Case, results: &[&Symbol]) -> Vec<String> {
        results
            .iter()
            .take(20)
            .filter(|s| case.avoid.iter().any(|a| s.name_text.contains(a.as_str())))
            .map(|s| s.name.clone())
            .collect()
    }

    #[test]
    fn search_quality() {
        let index = SymbolIndex::load("./media");
        let text = std::fs::read_to_string("tests/search-queries.yaml").unwrap();
        let cases: Vec<Case> = serde_yaml_ng::from_str(&text).unwrap();

        let (mut en_failed, mut ms_passed, mut ms_total, mut bad) = (vec![], 0, 0, vec![]);
        for case in &cases {
            let results = index.search(&case.q, 100);
            let ok = found(case, &results);
            if case.lang == "ms" {
                ms_total += 1;
                ms_passed += ok as usize;
            } else if !ok {
                en_failed.push(case.q.clone());
            }
            let names = unwanted(case, &results);
            if !names.is_empty() {
                bad.push(format!("{}: {:?}", case.q, names));
            }
            if !ok {
                println!("miss [{}] {}", case.lang, case.q);
            }
        }

        println!("Malay {ms_passed}/{ms_total}");
        assert!(en_failed.is_empty(), "English misses: {en_failed:?}");
        assert!(bad.is_empty(), "unwanted results: {bad:?}");
        assert!(ms_passed >= MS_FLOOR, "Malay fell to {ms_passed}");
    }

    #[test]
    fn short_words_stay_whole() {
        let index = SymbolIndex::load("./media");
        let names = |q| -> Vec<String> {
            index
                .search(q, 100)
                .iter()
                .map(|s| s.name.clone())
                .collect()
        };

        assert!(names("nak").is_empty());
        assert!(names("tak nak").is_empty());
        assert!(names("???").is_empty());
        assert!(names("appl").iter().any(|n| n.starts_with("apple")));
        assert!(names("berry").iter().any(|n| n.contains("strawberry")));
    }
}

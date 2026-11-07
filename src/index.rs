use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Shortest query that also matches its plural.
const MIN_PLURAL_LEN: usize = 3;
/// Shortest query that may match the start of a longer word ("appl" → apple).
/// Three letters is too loose: "nak" is Malay for "want", and must not reach
/// "naked".
const MIN_PREFIX_LEN: usize = 4;
/// Shortest query that may match inside a word ("berry" → strawberry).
const MIN_INSIDE_LEN: usize = 5;
/// Score of a word that is a whole Malay word or English tag. Words that mean
/// something in both languages ("air", "main") take turns from here up, so both
/// meanings show. "water" isn't one: it only appears inside "water heater".
const SHARED_WORD_SCORE: i32 = 60;
/// Lowest score for a whole-word match.
const WHOLE_WORD_SCORE: i32 = 50;
/// Malay words for each symbol, by path under media. Remove it and search is
/// English only.
const MALAY_WORDS_FILE: &str = "words-ms.yaml";

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub file: String,
    /// Name and tags as `padded()` text, built once at load.
    name_text: String,
    tag_texts: Vec<String>,
    malay_texts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SymbolIndex {
    symbols: Vec<Symbol>,
}

/// One search result before ranking.
struct Hit<'a> {
    symbol: &'a Symbol,
    score: i32,
    /// Where the Malay word sits in the symbol's list: the main word comes first.
    position: usize,
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

/// A search word or phrase, with the text `fit` looks for built once.
struct Query<'a> {
    text: &'a str,
    len: usize,
    words: String,
    plural: String,
    word_start: String,
}

impl<'a> Query<'a> {
    fn new(text: &'a str) -> Self {
        let len = text.chars().count();
        let words = format!(" {text} ");
        Query {
            text,
            len,
            // "dog" also finds "dogs", but "hi" must not find "his".
            plural: if len >= MIN_PLURAL_LEN {
                format!(" {text}s ")
            } else {
                words.clone()
            },
            words,
            word_start: format!(" {text}"),
        }
    }
}

fn fit(text: &str, query: &Query) -> Fit {
    if text == query.words {
        Fit::Exact
    } else if text.starts_with(&query.words) || text.starts_with(&query.plural) {
        Fit::WordsAtStart
    } else if text.contains(&query.words) || text.contains(&query.plural) {
        Fit::Words
    } else if query.len >= MIN_PREFIX_LEN && text.contains(&query.word_start) {
        Fit::Prefix
    } else if query.len >= MIN_INSIDE_LEN && text.contains(query.text) {
        Fit::Inside
    } else {
        Fit::None
    }
}

impl Symbol {
    fn new(name: String, file: String, tags: &[String], malay: &[String]) -> Self {
        Symbol {
            name_text: padded(&name),
            tag_texts: tags.iter().map(|tag| padded(tag)).collect(),
            malay_texts: malay.iter().map(|word| padded(word)).collect(),
            name,
            file,
        }
    }

    fn best_tag_fit(&self, query: &Query) -> Fit {
        self.tag_texts
            .iter()
            .map(|tag| fit(tag, query))
            .fold(Fit::None, |best, f| if f > best { f } else { best })
    }

    /// The best-fitting Malay word and its place in the list, earliest on a tie.
    fn best_malay_fit(&self, query: &Query) -> (Fit, usize) {
        self.malay_texts
            .iter()
            .enumerate()
            .map(|(position, word)| (fit(word, query), position))
            .fold(
                (Fit::None, 0),
                |best, next| {
                    if next.0 > best.0 {
                        next
                    } else {
                        best
                    }
                },
            )
    }

    fn english_score(&self, query: &Query) -> i32 {
        // Whole words beat word starts, and a name beats a tag at the same
        // level: "want" finds the tag "want" before the name "wanted".
        match (fit(&self.name_text, query), self.best_tag_fit(query)) {
            (Fit::Exact, _) => 100,
            (Fit::WordsAtStart, _) => 80,
            (Fit::Words, _) => 70,
            (_, Fit::Exact) => SHARED_WORD_SCORE,
            (_, Fit::WordsAtStart | Fit::Words) => WHOLE_WORD_SCORE,
            (Fit::Prefix, _) => 40,
            (_, Fit::Prefix) => 35,
            (Fit::Inside, _) => 30,
            (_, Fit::Inside) => 25,
            _ => 0,
        }
    }

    /// Malay words score the same as English tags.
    fn malay_score(fit: Fit) -> i32 {
        match fit {
            Fit::Exact => SHARED_WORD_SCORE,
            Fit::WordsAtStart | Fit::Words => WHOLE_WORD_SCORE,
            Fit::Prefix => 35,
            Fit::Inside => 25,
            Fit::None => 0,
        }
    }

    /// Several words: 10 for each whole word found anywhere in the symbol, 5
    /// for a part of one, so "nasi" prefers rice to "nasihat".
    fn partial_score(&self, terms: &[Query]) -> i32 {
        terms
            .iter()
            .map(|term| {
                let best = [
                    fit(&self.name_text, term),
                    self.best_tag_fit(term),
                    self.best_malay_fit(term).0,
                ]
                .into_iter()
                .fold(Fit::None, |best, f| if f > best { f } else { best });

                match best {
                    Fit::None => 0,
                    Fit::Inside | Fit::Prefix => 5,
                    _ => 10,
                }
            })
            .sum()
    }
}

/// Take turns between two ranked lists, skipping symbols already taken.
fn alternate<'a>(first: &[Hit<'a>], second: &[Hit<'a>]) -> Vec<&'a Symbol> {
    let mut merged: Vec<&Symbol> = Vec::new();
    for i in 0..first.len().max(second.len()) {
        for hit in [first.get(i), second.get(i)].into_iter().flatten() {
            if !merged.iter().any(|s| std::ptr::eq(*s, hit.symbol)) {
                merged.push(hit.symbol);
            }
        }
    }
    merged
}

fn load_malay_words(media_path: &Path) -> HashMap<String, Vec<String>> {
    let Ok(content) = std::fs::read_to_string(media_path.join(MALAY_WORDS_FILE)) else {
        tracing::warn!("no {} found, search is English only", MALAY_WORDS_FILE);
        return HashMap::new();
    };
    serde_yaml_ng::from_str(&content)
        .unwrap_or_else(|_| panic!("failed to parse {}", MALAY_WORDS_FILE))
}

impl SymbolIndex {
    pub fn load(media_dir: &str) -> Self {
        let mut symbols = Vec::new();

        let media_path = Path::new(media_dir);
        let malay_words = load_malay_words(media_path);
        let mut entries: Vec<_> = std::fs::read_dir(media_path)
            .expect("failed to read media directory")
            .flatten()
            .collect();
        // Tied results keep load order, so it must be the same on every machine.
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
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
                let path = format!("{}/{}", library, entry.file);
                let malay = malay_words.get(&path).map_or(&[][..], Vec::as_slice);
                let file = format!("media/{}", path);
                symbols.push(Symbol::new(entry.name, file, &entry.tags, malay));
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
        let terms: Vec<Query> = query
            .split(' ')
            .filter(|t| !t.is_empty())
            .map(Query::new)
            .collect();
        if terms.is_empty() {
            return Vec::new();
        }
        let phrase = Query::new(query);

        // Rank each language on its own, then let them share the top places.
        let (mut english, mut malay) = (Vec::new(), Vec::new());
        for symbol in &self.symbols {
            let english_score = symbol.english_score(&phrase);
            let (malay_fit, position) = symbol.best_malay_fit(&phrase);
            let malay_score = Symbol::malay_score(malay_fit);

            if malay_score > english_score {
                malay.push(Hit {
                    symbol,
                    score: malay_score,
                    position,
                });
            } else if english_score > 0 {
                english.push(Hit {
                    symbol,
                    score: english_score,
                    position: 0,
                });
            } else {
                let score = symbol.partial_score(&terms);
                if score > 0 {
                    english.push(Hit {
                        symbol,
                        score,
                        position: 0,
                    });
                }
            }
        }

        // Malay adds prefixes to words, so "beras" (rice) starts "berasap" (smoky)
        // and "berasal" (comes from). Once a whole word is found, drop those.
        let whole_word = |h: &Hit| h.score >= WHOLE_WORD_SCORE;
        if english.iter().chain(&malay).any(whole_word) {
            malay.retain(whole_word);
        }

        let rank = |hits: &mut Vec<Hit>| hits.sort_by_key(|h| (-h.score, h.position));
        rank(&mut english);
        rank(&mut malay);

        let top_count = |hits: &[Hit]| hits.partition_point(|h| h.score >= SHARED_WORD_SCORE);
        let (english_top, english_rest) = english.split_at(top_count(&english));
        let (malay_top, malay_rest) = malay.split_at(top_count(&malay));

        // The language with the better top match leads.
        let top_score = |hits: &[Hit]| hits.first().map_or(0, |h| h.score);
        let mut results = if top_score(malay_top) > top_score(english_top) {
            alternate(malay_top, english_top)
        } else {
            alternate(english_top, malay_top)
        };

        let mut rest: Vec<&Hit> = english_rest.iter().chain(malay_rest).collect();
        rest.sort_by_key(|h| (-h.score, h.position));
        results.extend(rest.into_iter().map(|h| h.symbol));

        results.truncate(limit);
        results
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
    const MS_FLOOR: usize = 96;

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

        assert!(names("nak").iter().all(|n| !n.contains("naked")));
        assert!(names("???").is_empty());
        assert!(names("appl").iter().any(|n| n.starts_with("apple")));
        assert!(names("berry").iter().any(|n| n.contains("strawberry")));
    }
}

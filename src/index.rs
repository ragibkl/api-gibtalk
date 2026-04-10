use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub file: String,
    pub tags: Vec<String>,
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

impl Symbol {
    fn score(&self, query: &str, terms: &[&str]) -> i32 {
        let name_lower = self.name.to_lowercase();

        // Exact name match
        if name_lower == *query {
            return 100;
        }

        // Name starts with query
        if name_lower.starts_with(query) {
            return 80;
        }

        // Name contains query
        if name_lower.contains(query) {
            return 60;
        }

        // Exact tag match
        for tag in &self.tags {
            let tag_lower = tag.to_lowercase();
            if tag_lower == *query {
                return 50;
            }
        }

        // Tag contains query
        for tag in &self.tags {
            let tag_lower = tag.to_lowercase();
            if tag_lower.contains(query) {
                return 30;
            }
        }

        // Partial: any search term matches name or tags
        let mut matched_terms = 0;
        for term in terms {
            if name_lower.contains(term) {
                matched_terms += 1;
                continue;
            }
            for tag in &self.tags {
                if tag.to_lowercase().contains(term) {
                    matched_terms += 1;
                    break;
                }
            }
        }

        if matched_terms > 0 {
            return 10 * matched_terms as i32;
        }

        0
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
            let yaml_path = entry.path().join("symbols.yaml");

            if !yaml_path.exists() {
                tracing::warn!("no symbols.yaml found in {}", library);
                continue;
            }

            let content =
                std::fs::read_to_string(&yaml_path).expect("failed to read symbols.yaml");
            let entries: Vec<SymbolEntry> =
                serde_yml::from_str(&content).expect("failed to parse symbols.yaml");

            for entry in entries {
                symbols.push(Symbol {
                    name: entry.name,
                    file: format!("media/{}/{}", library, entry.file),
                    tags: entry.tags,
                });
            }
        }

        tracing::info!("loaded {} symbols from {}", symbols.len(), media_dir);
        SymbolIndex { symbols }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<&Symbol> {
        let query_lower = query.to_lowercase();
        let terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(&Symbol, i32)> = self
            .symbols
            .iter()
            .filter_map(|symbol| {
                let score = symbol.score(&query_lower, &terms);
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

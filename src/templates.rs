use serde::{Deserialize, Serialize};
use std::path::Path;
use url::Url;

// Ready-made word sets the app offers. Stored as YAML in ./templates (the same
// format as the app's backup files), with picture paths relative to ./media so
// they don't name a host. Served as JSON with full URLs built from BASE_URL.

#[derive(Debug, Clone, Deserialize)]
struct IndexEntry {
    id: String,
    name: String,
    description: String,
    author: String,
    language: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Word {
    label: String,
    language: Option<String>,
    uri: String,
    children: Option<Vec<Word>>,
}

#[derive(Debug, Clone)]
pub struct Template {
    entry: IndexEntry,
    words: Vec<Word>,
}

#[derive(Debug, Clone)]
pub struct Templates {
    templates: Vec<Template>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub language: String,
    pub words: usize,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordOutput {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<WordOutput>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOutput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub language: String,
    pub words: Vec<WordOutput>,
}

fn count_words(words: &[Word]) -> usize {
    words
        .iter()
        .map(|w| 1 + w.children.as_deref().map_or(0, count_words))
        .sum()
}

// Panics on a picture that isn't in ./media, so a broken template can't go live.
fn check_pictures(id: &str, words: &[Word], media_dir: &Path) {
    for word in words {
        if !media_dir.join(&word.uri).is_file() {
            panic!("template {}: picture not found: {}", id, word.uri);
        }
        if let Some(children) = &word.children {
            check_pictures(id, children, media_dir);
        }
    }
}

fn word_output(word: &Word, base_url: &Url) -> WordOutput {
    let uri = base_url
        .join(&format!("media/{}", word.uri))
        .map(|u| u.to_string())
        .unwrap_or_default();
    WordOutput {
        label: word.label.clone(),
        language: word.language.clone(),
        uri,
        children: word
            .children
            .as_ref()
            .map(|c| c.iter().map(|w| word_output(w, base_url)).collect()),
    }
}

impl Templates {
    pub fn load(templates_dir: &str, media_dir: &str) -> Self {
        let dir = Path::new(templates_dir);
        let media = Path::new(media_dir);

        let index = std::fs::read_to_string(dir.join("index.yaml"))
            .expect("failed to read templates/index.yaml");
        let entries: Vec<IndexEntry> =
            serde_yml::from_str(&index).expect("failed to parse templates/index.yaml");

        let templates = entries
            .into_iter()
            .map(|entry| {
                let path = dir.join(format!("{}.yaml", entry.id));
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("failed to read {}", path.display()));
                let words: Vec<Word> = serde_yml::from_str(&content)
                    .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));
                check_pictures(&entry.id, &words, media);
                Template { entry, words }
            })
            .collect::<Vec<_>>();

        tracing::info!(
            "loaded {} templates from {}",
            templates.len(),
            templates_dir
        );
        Templates { templates }
    }

    pub fn list(&self, base_url: &Url) -> Vec<TemplateSummary> {
        self.templates
            .iter()
            .map(|t| TemplateSummary {
                id: t.entry.id.clone(),
                name: t.entry.name.clone(),
                description: t.entry.description.clone(),
                author: t.entry.author.clone(),
                language: t.entry.language.clone(),
                words: count_words(&t.words),
                url: base_url
                    .join(&format!("api/templates/{}", t.entry.id))
                    .map(|u| u.to_string())
                    .unwrap_or_default(),
            })
            .collect()
    }

    pub fn get(&self, id: &str, base_url: &Url) -> Option<TemplateOutput> {
        let t = self.templates.iter().find(|t| t.entry.id == id)?;
        Some(TemplateOutput {
            id: t.entry.id.clone(),
            name: t.entry.name.clone(),
            description: t.entry.description.clone(),
            author: t.entry.author.clone(),
            language: t.entry.language.clone(),
            words: t.words.iter().map(|w| word_output(w, base_url)).collect(),
        })
    }
}

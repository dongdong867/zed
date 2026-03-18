use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use gpui::{App, AppContext, AsyncApp, Entity, Task, WeakEntity};
use language::{Buffer, Language, LanguageId, LanguageRegistry, Node, PLAIN_TEXT};
#[cfg(feature = "magika")]
use magika::ContentType;
use streaming_iterator::StreamingIterator;

const SAMPLE_MIN_BYTES: usize = 50;
const SAMPLE_MAX_BYTES: usize = 8192;
const DEBOUNCE_MS: u64 = 300;
const SCORE_THRESHOLD: f64 = 0.1;
const COMMON_LANGUAGE_NAMES: &[&str] = &[
    "JSON", "YAML", "HTML", "Markdown", "Rust", "Python", "JavaScript", "TypeScript", "Go", "C",
    "C++", "Shell Script", "Ruby", "CSS",
];

pub struct LanguageDetector {
    language_registry: Arc<LanguageRegistry>,
    detection_task: Option<Task<()>>,
    registry_watcher: Option<Task<()>>,
    preload_task: Option<Task<()>>,
    last_detected: Arc<Mutex<Option<LanguageId>>>,
}

impl LanguageDetector {
    pub fn new(language_registry: Arc<LanguageRegistry>) -> Self {
        Self {
            language_registry,
            detection_task: None,
            registry_watcher: None,
            preload_task: None,
            last_detected: Arc::new(Mutex::new(None)),
        }
    }

    pub fn schedule(&mut self, buffer: Entity<Buffer>, cx: &mut App) {
        let buffer_ref = buffer.read(cx);
        if !self.should_detect(buffer_ref) {
            return;
        }

        if self.preload_task.is_none() {
            self.preload_task = Some(Self::spawn_preload(self.language_registry.clone(), cx));
        }

        if self.registry_watcher.is_none() {
            self.registry_watcher = Some(self.spawn_registry_watcher(
                buffer.downgrade(),
                self.language_registry.clone(),
                cx,
            ))
        }

        let registry = self.language_registry.clone();
        let weak_buffer = buffer.downgrade();
        let last_detected = self.last_detected.clone();
        self.detection_task = Some(cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(DEBOUNCE_MS))
                .await;
            Self::run_detection(weak_buffer, registry, last_detected, cx).await;
        }))
    }

    pub fn stop(&mut self) {
        self.detection_task = None;
        self.registry_watcher = None;
        self.preload_task = None;
        if let Ok(mut last) = self.last_detected.lock() {
            *last = None;
        }
    }

    fn should_detect(&self, buffer: &Buffer) -> bool {
        if buffer.file().is_some() {
            return false;
        }

        if buffer.len() < SAMPLE_MIN_BYTES {
            return false;
        }

        let current_language = buffer.language();

        let is_plain_text = current_language
            .map(|lang| lang.id() == PLAIN_TEXT.id())
            .unwrap_or(true);

        if is_plain_text {
            return true;
        }

        // Allow re-detection when the current language was set by us.
        if let Ok(last) = self.last_detected.lock() {
            if let Some(detected_id) = *last {
                if let Some(lang) = current_language {
                    return lang.id() == detected_id;
                }
            }
        }

        false
    }

    fn spawn_preload(registry: Arc<LanguageRegistry>, cx: &mut App) -> Task<()> {
        cx.background_spawn(async move {
            for name in COMMON_LANGUAGE_NAMES {
                registry.language_for_name(name).await.ok();
            }
        })
    }

    fn spawn_registry_watcher(
        &self,
        buffer: WeakEntity<Buffer>,
        registry: Arc<LanguageRegistry>,
        cx: &mut App,
    ) -> Task<()> {
        let last_detected = self.last_detected.clone();
        let mut subscription = registry.subscribe();
        cx.spawn(async move |cx| {
            while subscription.next().await.is_some() {
                let Ok(should_run) = buffer.read_with(cx, |b, _| {
                    if b.file().is_some() || b.len() < SAMPLE_MIN_BYTES {
                        return false;
                    }
                    let current_language = b.language();
                    let is_plain_text = current_language
                        .map(|lang| lang.id() == PLAIN_TEXT.id())
                        .unwrap_or(true);
                    if is_plain_text {
                        return true;
                    }
                    // Only re-detect if we originally set the current language.
                    if let Ok(last) = last_detected.lock() {
                        if let Some(detected_id) = *last {
                            return current_language
                                .is_some_and(|lang| lang.id() == detected_id);
                        }
                    }
                    false
                }) else {
                    break;
                };

                if !should_run {
                    break;
                }

                Self::run_detection(
                    buffer.clone(),
                    registry.clone(),
                    last_detected.clone(),
                    cx,
                )
                .await;
            }
        })
    }

    async fn run_detection(
        buffer: WeakEntity<Buffer>,
        registry: Arc<LanguageRegistry>,
        last_detected: Arc<Mutex<Option<LanguageId>>>,
        cx: &mut AsyncApp,
    ) {
        let snapshot = match buffer.read_with(cx, |b, _| b.snapshot()) {
            Ok(s) => s,
            Err(_) => return,
        };

        if snapshot.len() < SAMPLE_MIN_BYTES {
            return;
        }
        let sample: Vec<u8> = snapshot
            .text_for_range(0..snapshot.len().min(SAMPLE_MAX_BYTES))
            .collect::<String>()
            .into_bytes();

        let candidates = registry.loaded_languages_with_grammars();
        if candidates.is_empty() {
            return;
        }

        let detected: Option<Arc<Language>> = cx
            .background_spawn(async move { best_match(&sample, &candidates) })
            .await;

        let Some(language) = detected else { return };

        if let Ok(mut last) = last_detected.lock() {
            *last = Some(language.id());
        }

        buffer
            .update(cx, |b, cx| b.set_language(Some(language), cx))
            .ok();
    }
}

/// Picks the best-matching language for `sample` from `candidates`.
///
/// Scores each candidate using a combination of parse quality (error-free
/// parsing) and highlight query coverage (how many semantic tokens the
/// language's queries recognize). The highest-scoring candidate wins.
pub fn best_match(sample: &[u8], candidates: &[Arc<Language>]) -> Option<Arc<Language>> {
    let mut scored: Vec<(Arc<Language>, f64)> = candidates
        .iter()
        .filter_map(|language| {
            let score = score_language(language, sample);
            if score > SCORE_THRESHOLD {
                Some((language.clone(), score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().next().map(|(lang, _)| lang)
}

/// Scores how well `language` explains `text` by combining three signals:
///
///   1. **Parse quality** — fraction of bytes NOT inside ERROR nodes. A grammar
///      that truly matches the language will parse with minimal errors.
///   2. **Highlight coverage** — fraction of bytes captured by highlight queries.
///      The right grammar's queries will match keywords, strings, operators, etc.
///   3. **Capture diversity** — number of distinct highlight capture types found.
///      Real matches produce many capture types (keyword, string, type, operator);
///      false positives often hit only 1–2 types.
pub fn score_language(language: &Language, text: &[u8]) -> f64 {
    let grammar = match language.grammar() {
        Some(g) => g,
        None => return 0.0,
    };

    let text_len = text.len();
    if text_len == 0 {
        return 0.0;
    }

    let Some(tree) = language::with_parser(|parser| {
        if parser.set_language(&grammar.ts_language).is_err() {
            return None;
        }
        parser.parse(text, None)
    }) else {
        return 0.0;
    };

    let error_bytes = count_error_bytes(tree.root_node(), text_len);
    let parse_quality = 1.0 - (error_bytes as f64 / text_len as f64);

    // Reject grammars that produce >50% errors — they clearly don't understand
    // the content.
    if parse_quality < 0.5 {
        return 0.0;
    }

    let (coverage, diversity) = match &grammar.highlights_config {
        Some(config) => language::with_query_cursor(|cursor| {
            cursor.set_byte_range(0..text_len);

            let mut covered_ranges: Vec<std::ops::Range<usize>> = Vec::new();
            let mut seen_capture_names = std::collections::HashSet::new();

            let mut captures = cursor.captures(&config.query, tree.root_node(), text);
            while let Some((query_match, capture_idx)) = captures.next() {
                let capture = &query_match.captures[*capture_idx];
                covered_ranges.push(capture.node.byte_range());
                seen_capture_names.insert(config.query.capture_names()[capture.index as usize]);
            }

            if covered_ranges.is_empty() {
                return (0.0, 0.0);
            }

            // Merge overlapping ranges before summing to avoid double-counting bytes
            // covered by nested capture nodes (e.g., a function node that also
            // contains keyword and identifier children, all captured separately).
            covered_ranges.sort_unstable_by_key(|r| r.start);
            let covered_bytes: usize = {
                let mut total = 0usize;
                let mut max_end = 0usize;
                for range in &covered_ranges {
                    let start = range.start.max(max_end);
                    if start < range.end {
                        total += range.end - start;
                        max_end = range.end;
                    }
                }
                total
            };

            // Coverage: fraction of text bytes covered by captures.
            let cov = (covered_bytes as f64 / text_len as f64).min(1.0);

            // Diversity: distinct capture types, normalized. Real language matches
            // typically produce 5–15+ distinct capture types (keyword, string,
            // number, operator, type, function, property, etc.). We normalize by
            // dividing by 10 and capping at 1.0, so ≥10 distinct types = full score.
            let div = (seen_capture_names.len() as f64 / 10.0).min(1.0);

            (cov, div)
        }),
        None => (0.0, 0.0),
    };

    // Final score: weighted combination of all three signals.
    // - parse_quality (30%): penalizes grammars that produce many errors
    // - coverage (30%): rewards grammars whose queries match lots of content
    // - diversity (40%): rewards grammars that recognize many semantic categories
    //   (this is the strongest discriminator — the right grammar finds keywords,
    //   strings, types, operators, functions, etc., while wrong grammars only match
    //   a few generic patterns)
    parse_quality * 0.3 + coverage * 0.3 + diversity * 0.4
}

fn count_error_bytes(node: Node, text_len: usize) -> usize {
    let mut error_bytes = 0;
    let mut cursor = node.walk();
    let mut depth = 0;

    loop {
        let current = cursor.node();
        if current.is_error() || current.is_missing() {
            error_bytes += current.byte_range().len();
            // Don't descend into error nodes — already counted their full range.
        } else if cursor.goto_first_child() {
            depth += 1;
            continue;
        }

        while !cursor.goto_next_sibling() {
            if depth == 0 {
                return error_bytes.min(text_len);
            }
            cursor.goto_parent();
            depth -= 1;
        }
    }
}

#[cfg(feature = "magika")]
fn content_type_to_language_name(content_type: ContentType) -> Option<&'static str> {
    match content_type {
        ContentType::C => Some("C"),
        ContentType::Cpp => Some("C++"),
        ContentType::Css => Some("CSS"),
        ContentType::Diff => Some("Diff"),
        ContentType::Go => Some("Go"),
        ContentType::Html => Some("HTML"),
        ContentType::Javascript => Some("JavaScript"),
        ContentType::Json => Some("JSON"),
        ContentType::Markdown => Some("Markdown"),
        ContentType::Python => Some("Python"),
        ContentType::Ruby => Some("Ruby"),
        ContentType::Rust => Some("Rust"),
        ContentType::Shell => Some("Shell Script"),
        ContentType::Sql => Some("SQL"),
        ContentType::Swift => Some("Swift"),
        ContentType::Toml => Some("TOML"),
        ContentType::Typescript => Some("TypeScript"),
        ContentType::Xml => Some("XML"),
        ContentType::Yaml => Some("YAML"),
        _ => None,
    }
}

#[cfg(feature = "magika")]
pub struct MagikaDetector {
    session: magika::Session,
}

#[cfg(feature = "magika")]
impl MagikaDetector {
    pub fn new() -> anyhow::Result<Self> {
        let session = magika::Session::new().map_err(anyhow::Error::new)?;
        Ok(Self { session })
    }

    pub fn detect_language_name(&mut self, text: &[u8]) -> Option<&'static str> {
        let file_type = self.session.identify_content_sync(text).ok()?;
        let content_type = file_type.content_type()?;
        content_type_to_language_name(content_type)
    }
}

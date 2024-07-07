#![allow(unused)]

use std::{collections::HashMap, mem};

use indexmap::IndexMap;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

#[derive(Debug, Clone)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
}

impl Span {
    fn to_bytes(&self) -> [u8; 4] {
        let mut bytes = [0; 4];
        let offset_bytes = self.offset.to_le_bytes();
        bytes[0..3].clone_from_slice(&offset_bytes[0..3]);
        bytes[3] = self.length.to_le_bytes()[0];
        bytes
    }
}

pub fn bolden(text: &str, spans: &[Span]) -> String {
    let mut spans = spans.to_owned();
    spans.sort_by_key(|s| s.offset);

    let mut text = text.to_owned();
    for (i, span) in spans.iter().enumerate() {
        let offset = i * 7;
        text.insert_str(span.offset + offset, "<b>");
        text.insert_str(span.offset + offset + 3 + span.length, "</b>");
    }
    text
}

/// The result of a call to `Index::words`.
pub struct SearchResults {
    /// A mapping of section name to word instances.
    pub spans: HashMap<String, (bool, Vec<Span>)>,

    /// A mapping of section name to scores.
    pub scores: HashMap<String, f64>,
}

impl SearchResults {
    /// The total score for this index across sections.
    pub fn total_score(&self) -> f64 {
        self.scores.values().sum()
    }

    /// Get the score and spans for one section.
    pub fn for_section(&self, section: &str) -> Option<(f64, bool, &[Span])> {
        self.spans.get(section).and_then(|spans| {
            self.scores
                .get(section)
                .map(|&score| (score, spans.0, spans.1.as_slice()))
        })
    }

    /// Whether all the words in the query were found at least once
    /// in any section.
    pub fn all_words_found(&self) -> bool {
        self.spans.values().any(|spans| spans.0)
    }

    /// Whether all the words in the query were found at least once
    /// in the specified section.
    pub fn all_words_found_for_section(&self, section: &str) -> bool {
        self.spans.get(section).is_some_and(|spans| spans.0)
    }

    /// The total number of words in all sections.
    pub fn total_hit_count(&self) -> usize {
        self.spans.values().map(|spans| spans.1.len()).sum()
    }
}

#[derive(Debug)]
pub struct Index {
    sections: Vec<Section>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            sections: Vec::new(),
        }
    }

    pub fn add_section(&mut self, name: String, processed_text: &str) -> usize {
        let stemmer = Stemmer::create(Algorithm::English);

        let processed_text = processed_text
            .replace("<i>", "***")
            .replace("</i>", "****")
            .replace("<I>", "***")
            .replace("</I>", "****")
            .replace("/note", "*****")
            .replace("/aside", "******")
            .replace("/end", "****");

        let img_regex = Regex::new(r"/img [\S]+").unwrap();
        let mut new_text = processed_text.clone();
        for instance in img_regex.find_iter(&processed_text) {
            new_text = format!("{}{}{}",
                &new_text[0..instance.start()],
                "*".repeat(instance.len()),
                &new_text[instance.end()..]);
        }
        let processed_text = new_text;

        let word_regex = Regex::new(r"[\p{L}\d’]+").unwrap();

        let mut section = Section {
            name,
            words: HashMap::new(),
        };

        let mut count = 0;
        for instance in word_regex.find_iter(&processed_text) {
            count += 1;
            let word = instance.as_str().to_lowercase().replace('’', "'");
            let word = stemmer.stem(&word);

            let span = Span {
                length: instance.len(),
                offset: instance.start(),
            };

            section
                .words
                .entry(word.into_owned())
                .or_insert_with(Vec::new)
                .push(span);
        }

        self.sections.push(section);

        count
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for section in &self.sections {
            bytes.extend(section.to_bytes().into_iter());
        }

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut sections = Vec::new();

        let mut bytes = bytes.iter();
        let mut current_section = None;
        while let Some(&c) = bytes.next() {
            if c == b'$' {
                let mut section_name_bytes = Vec::new();
                loop {
                    let &next_byte = bytes.next().expect("section name incomplete");
                    if next_byte == b'$' {
                        break;
                    };
                    section_name_bytes.push(next_byte);
                }
                let section_name =
                    String::from_utf8(section_name_bytes).expect("section name is not valid utf-8");
                let completed_section = mem::replace(
                    &mut current_section,
                    Some(Section {
                        name: section_name,
                        words: HashMap::new(),
                    }),
                );

                // Finalize completed section.
                if let Some(completed_section) = completed_section {
                    sections.push(completed_section);
                }
            } else {
                let Some(ref mut current_section) = current_section else {
                    panic!("word encountered with no section declared");
                };

                // Get the next word.
                let mut word_bytes = vec![c];
                loop {
                    let &next_byte = bytes.next().expect("word incomplete");
                    if next_byte == b' ' {
                        break;
                    }
                    word_bytes.push(next_byte);
                }
                let word = String::from_utf8(word_bytes).expect("word is not valid utf-8");

                // Collect instances.
                let span_count_bytes: Vec<_> = bytes.by_ref().take(2).map(|b| *b).collect();
                let span_count_bytes = span_count_bytes
                    .try_into()
                    .expect("invalid span count byets");
                let span_count = u16::from_le_bytes(span_count_bytes) as usize;

                let mut spans = Vec::with_capacity(span_count);
                for _ in 0..span_count {
                    let span_bytes: Vec<_> = bytes.by_ref().take(4).map(|b| *b).collect();
                    let mut offset_bytes = [0; 8];
                    offset_bytes[0..3].clone_from_slice(&span_bytes[0..3]);
                    let offset = usize::from_le_bytes(offset_bytes);
                    let length = span_bytes[3] as usize;
                    spans.push(Span { offset, length });
                }

                // Register word.
                current_section.words.insert(word, spans);
            }
        }
        if let Some(current_section) = current_section {
            sections.push(current_section);
        }

        Index { sections }
    }

    /// Returns a "score" and a mapping of search section to spans
    /// that match the words appearing in that section.
    pub fn words(&self, words: &[&str]) -> SearchResults {
        let mut spans = HashMap::new();
        let mut scores = HashMap::new();

        for section in &self.sections {
            let word_results = section.words(words);
            spans.insert(section.name.clone(), (word_results.1, word_results.2));
            scores.insert(section.name.clone(), word_results.0);
        }

        SearchResults { spans, scores }
    }

    /// Returns the total number of words in all sections.
    pub fn total_word_count(&self) -> usize {
        self.sections.iter().map(|s| s.word_count()).sum()
    }

    /// Returns a mapping of all words and their counts.
    pub fn all_words(&self) -> HashMap<String, usize> {
        self.sections
            .iter()
            .flat_map(|s| s.words.iter())
            .fold(HashMap::new(), |mut map, (word, spans)| {
                *map.entry(word.to_owned()).or_insert(0) += spans.len();
                map
            })
    }
}

impl Default for Index {
    fn default() -> Self {
        Self {
            sections: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Section {
    name: String,
    words: HashMap<String, Vec<Span>>,
}

impl Section {
    fn to_bytes(&self) -> Vec<u8> {
        let words_length = self
            .words
            .iter()
            .map(|(word, spans)| {
                // Add 1 for the space character.
                let word_length = word.len() + 1;

                // Add 2 for the bytes for the count of spans.
                let spans_length = 2 + 4 * spans.len();

                word_length + spans_length
            })
            .reduce(|acc, l| acc + l)
            .unwrap_or(0);
        let bytes_length = 2 + self.name.len() + words_length;
        let mut bytes = Vec::with_capacity(bytes_length);

        bytes.push(b'$');
        bytes.extend(self.name.bytes());
        bytes.push(b'$');

        for (word, spans) in &self.words {
            bytes.extend(word.bytes());
            bytes.push(b' ');

            let count_bytes = spans.len().to_le_bytes();
            bytes.extend(count_bytes[0..2].iter());

            for span in spans {
                bytes.extend(span.to_bytes().into_iter())
            }
        }

        bytes
    }

    /// Returns a "score" and a list of spans containing any of the
    /// specified words.
    pub fn words(&self, words: &[&str]) -> (f64, bool, Vec<Span>) {
        let mut score = 0.0;
        let word_count = self.word_count() as f64;
        let stemmer = Stemmer::create(Algorithm::English);

        // First collect the words.
        let words: IndexMap<_, _> = words
            .iter()
            .map(|&word| {
                let w = word.to_lowercase().replace('’', "'");
                let word = stemmer.stem(&w).into_owned();

                let spans = self
                    .words
                    .get(&word)
                    .cloned()
                    .unwrap_or_else(Vec::new);

                (word, spans)
            })
            .collect();

        let all_words_exist = words.values().all(|spans| !spans.is_empty());

        // Score each word.
        for (word, spans) in &words {
            for span in spans {
                // Start with a base score of 1 for each instance of the word.
                let mut word_score = 1.0;

                // If there were other words to search for, award bonuses for proximity.
                for (otherword, otherspans) in &words {
                    if word == otherword {
                        continue;
                    }

                    let mut bonus_multiplier = 1.0;

                    for otherspan in otherspans {
                        let distance = (span.offset as f64 - otherspan.offset as f64).abs();
                        let bonus = 110.0 / 1.2_f64.powf(distance);

                        // A distance of 0 is worth 100 points.
                        // A distance of 61 characters is worth 50.
                        // A distance of 241 characters is worth 0.
                        if bonus > 0.0 {
                            bonus_multiplier += bonus;
                        }
                    }

                    // More words in the query amounts to compounding score bonus.
                    word_score *= bonus_multiplier;
                }
                score += word_score / word_count;
            }
        }

        let spans = words.into_values().flatten().collect();
        (score, all_words_exist, spans)
    }

    /// The total number of captured words in this section.
    pub fn word_count(&self) -> usize {
        self.words.values().map(|w| w.len()).sum()
    }
}

use std::{collections::HashMap, mem};

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
            .replace('’', "'");
        let word_regex = Regex::new(r"[\p{L}\d\']+").unwrap();

        let mut section = Section {
            name,
            words: HashMap::new(),
        };

        let mut count = 0;
        for instance in word_regex.find_iter(&processed_text) {
            count += 1;
            let word = instance.as_str().to_lowercase();
            let word = stemmer.stem(&word);

            let span = Span {
                length: instance.len(),
                offset: instance.start(),
            };

            section
                .words
                .entry(word.into_owned())
                .or_insert(Vec::new())
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

    pub fn word(&self, word: &str) -> HashMap<String, Vec<Span>> {
        let stemmer = Stemmer::create(Algorithm::English);
        let word = word.to_lowercase().replace('’', "'");
        let word = stemmer.stem(&word).into_owned();

        let mut map = HashMap::new();

        for section in &self.sections {
            let spans = section
                .words
                .get(&word)
                .map(|spans| spans.clone())
                .unwrap_or(Vec::new());
            map.insert(section.name.clone(), spans);
        }

        map
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
}

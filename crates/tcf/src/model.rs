use unicode_normalization::{UnicodeNormalization, is_nfc};
use codec_common::{CodecError, Result};

/// Unicode text modeling for TCF compression
pub struct TextModel {
    /// Current context for prediction
    context: Vec<char>,
    /// Maximum context length
    max_order: usize,
    /// Character frequency tables by context
    char_freqs: std::collections::HashMap<Vec<char>, std::collections::HashMap<char, u32>>,
    /// Escape character frequency
    escape_freq: u32,
}

impl TextModel {
    pub fn new(max_order: usize) -> Self {
        Self {
            context: Vec::new(),
            max_order,
            char_freqs: std::collections::HashMap::new(),
            escape_freq: 1,
        }
    }

    /// Normalize Unicode text to NFC form
    pub fn normalize_text(text: &str) -> String {
        if is_nfc(text) {
            text.to_string()
        } else {
            text.nfc().collect()
        }
    }

    /// Update the model with a new character
    pub fn update(&mut self, ch: char) {
        // Update frequencies for current context
        let context_freqs = self.char_freqs.entry(self.context.clone()).or_insert_with(std::collections::HashMap::new);
        *context_freqs.entry(ch).or_insert(0) += 1;

        // Update context
        self.context.push(ch);
        if self.context.len() > self.max_order {
            self.context.remove(0);
        }
    }

    /// Get character frequencies for current context
    pub fn get_frequencies(&self, alphabet: &[char]) -> Vec<u32> {
        let mut freqs = Vec::with_capacity(alphabet.len() + 1); // +1 for escape

        if let Some(context_freqs) = self.char_freqs.get(&self.context) {
            for &ch in alphabet {
                freqs.push(*context_freqs.get(&ch).unwrap_or(&0));
            }
        } else {
            // No context found, use uniform distribution
            freqs = vec![1; alphabet.len()];
        }

        // Add escape frequency
        freqs.push(self.escape_freq);
        freqs
    }

    /// Get the symbol index for a character
    pub fn get_symbol_index(&self, ch: char, alphabet: &[char]) -> Option<usize> {
        alphabet.iter().position(|&c| c == ch)
    }

    /// Reset context
    pub fn reset_context(&mut self) {
        self.context.clear();
    }
}

/// Build a character alphabet from text
pub fn build_alphabet(text: &str) -> Vec<char> {
    let mut chars: Vec<char> = text.chars().collect::<std::collections::HashSet<_>>().into_iter().collect();
    chars.sort();
    chars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let text = "café"; // e with acute accent
        let normalized = TextModel::normalize_text(text);
        assert!(is_nfc(&normalized));
    }

    #[test]
    fn test_build_alphabet() {
        let text = "hello world";
        let alphabet = build_alphabet(text);
        assert!(alphabet.contains(&'h'));
        assert!(alphabet.contains(&' '));
        assert_eq!(alphabet.len(), 8); // h, e, l, o, space, w, r, d
    }

    #[test]
    fn test_text_model() {
        let mut model = TextModel::new(2);
        model.update('h');
        model.update('e');
        model.update('l');
        
        assert_eq!(model.context, vec!['e', 'l']);
    }
}
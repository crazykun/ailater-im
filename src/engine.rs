//! Input method engine core
//!
//! Main engine that coordinates pinyin parsing, dictionary lookup, and AI prediction.

use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::config::Config;
use crate::dictionary::Dictionary;
use crate::ffi::{FcitxInputContext, FcitxInstance, IMReturnValue, KeyState, KeySym};
use crate::model::{create_model_client, ModelBackend, PredictionSource};
use crate::pinyin::{get_candidates, get_initial_map, FuzzyPinyinMatcher, PinyinParser};

/// Input state for each input context
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current preedit string (pinyin input)
    pub preedit: String,
    /// Current cursor position in preedit
    pub cursor_pos: usize,
    /// Selected candidates
    pub candidates: Vec<Candidate>,
    /// Current candidate page
    pub current_page: usize,
    /// Currently selected candidate index within current page
    pub selected_index: usize,
    /// Committed text (context for AI)
    pub context: String,
    /// Is the input method active
    pub is_active: bool,
    /// Text to be committed (cleared after being read)
    pub commit_text: String,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            preedit: String::new(),
            cursor_pos: 0,
            candidates: Vec::new(),
            current_page: 0,
            selected_index: 0,
            context: String::new(),
            is_active: false,
            commit_text: String::new(),
        }
    }
}

/// Candidate word
#[derive(Debug, Clone)]
pub struct Candidate {
    /// The candidate text
    pub text: String,
    /// Pinyin for this candidate
    pub pinyin: String,
    /// Confidence score
    pub score: f32,
    /// Source of this candidate
    pub source: PredictionSource,
}

/// The main input method engine
pub struct InputEngine {
    /// Configuration
    config: Config,
    /// Pinyin parser
    pinyin_parser: PinyinParser,
    /// Fuzzy pinyin matcher
    fuzzy_matcher: FuzzyPinyinMatcher,
    /// Dictionary
    dictionary: Arc<Dictionary>,
    /// AI model client
    model: Box<dyn ModelBackend>,
    /// Input states per context (key: ic pointer)
    states: RwLock<HashMap<usize, InputState>>,
    /// Commit counter for auto-saving dictionary
    commit_counter: Arc<AtomicUsize>,
}

impl InputEngine {
    /// Create a new input engine
    pub fn new(config: Config) -> Self {
        let dictionary = Arc::new(Dictionary::new(config.dictionary.clone()));
        let model = create_model_client(config.model.clone());

        Self {
            config,
            pinyin_parser: PinyinParser::new(),
            fuzzy_matcher: FuzzyPinyinMatcher::new(),
            dictionary,
            model,
            states: RwLock::new(HashMap::new()),
            commit_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get or create input state for an input context
    fn get_state(&self, ic: *mut FcitxInputContext) -> InputState {
        let states = self.states.read();
        let key = ic as usize;
        states.get(&key).cloned().unwrap_or_default()
    }

    /// Update input state for an input context
    fn update_state(&self, ic: *mut FcitxInputContext, state: InputState) {
        let mut states = self.states.write();
        let key = ic as usize;
        states.insert(key, state);
    }

    /// Handle key event
    pub fn handle_key(
        &self,
        _instance: *mut FcitxInstance,
        ic: *mut FcitxInputContext,
        keysym: u32,
        _keycode: u32,
        state: u32,
        is_release: bool,
    ) -> IMReturnValue {
        if is_release {
            return IMReturnValue::Forward;
        }

        let key = KeySym::from_raw(keysym);
        let key_state = KeyState(state);
        let mut input_state = self.get_state(ic);

        // Handle modifier keys
        if key_state.has_ctrl() || key_state.has_alt() || key_state.has_super() {
            return IMReturnValue::Forward;
        }

        // Handle different key types
        // Left/Right: switch selected candidate
        // Up/Down: page up/down
        let result = match key {
            KeySym::BackSpace => self.handle_backspace(&mut input_state),
            KeySym::Return => self.handle_return(&mut input_state),
            KeySym::Escape => self.handle_escape(&mut input_state),
            KeySym::Space => self.handle_space(&mut input_state),
            KeySym::Minus => self.handle_page_up(&mut input_state),
            KeySym::Equal => self.handle_page_down(&mut input_state),
            KeySym::Plus => self.handle_page_down(&mut input_state),
            KeySym::Left => self.handle_left(&mut input_state),
            KeySym::Right => self.handle_right(&mut input_state),
            KeySym::Up => self.handle_page_up(&mut input_state),
            KeySym::Down => self.handle_page_down(&mut input_state),
            _ => {
                // Handle letter keys for pinyin input
                if key.is_letter(keysym) {
                    let ch = (keysym as u8) as char;
                    self.handle_letter(&mut input_state, ch)
                } else if key.is_number(keysym) {
                    let num = (keysym - 0x30) as usize;
                    self.handle_number(&mut input_state, num)
                } else if key.is_printable(keysym) {
                    // Handle punctuation
                    let ch = (keysym as u8) as char;
                    self.handle_punctuation(&mut input_state, ch)
                } else {
                    IMReturnValue::Forward
                }
            }
        };

        self.update_state(ic, input_state);
        result
    }

    /// Handle letter input
    fn handle_letter(&self, state: &mut InputState, ch: char) -> IMReturnValue {
        // Check max length
        if state.preedit.len() >= self.config.input.max_preedit_length {
            return IMReturnValue::Consume;
        }

        state.preedit.push(ch.to_ascii_lowercase());
        state.cursor_pos = state.preedit.len();
        state.is_active = true;

        // Update candidates
        self.update_candidates(state);

        IMReturnValue::Consume
    }

    /// Handle number key (for candidate selection)
    fn handle_number(&self, state: &mut InputState, num: usize) -> IMReturnValue {
        if !state.is_active || state.candidates.is_empty() {
            return IMReturnValue::Forward;
        }

        let page_start = state.current_page * self.config.input.page_size;
        let candidate_idx = page_start + num - 1; // 1-based to 0-based

        if candidate_idx < state.candidates.len() {
            let candidate = state.candidates[candidate_idx].clone();
            self.commit_candidate(state, &candidate);
            return IMReturnValue::Consume;
        }

        IMReturnValue::Forward
    }

    /// Handle punctuation
    fn handle_punctuation(&self, state: &mut InputState, _ch: char) -> IMReturnValue {
        if state.is_active && !state.preedit.is_empty() {
            // Commit first candidate if auto-commit is enabled
            if self.config.input.auto_commit_on_punctuation {
                if let Some(candidate) = state.candidates.first().cloned() {
                    self.commit_candidate(state, &candidate);
                }
            }
        }

        // Forward the punctuation
        IMReturnValue::Forward
    }

    /// Handle backspace
    fn handle_backspace(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.preedit.is_empty() {
            return IMReturnValue::Forward;
        }

        state.preedit.pop();
        state.cursor_pos = state.preedit.len();

        if state.preedit.is_empty() {
            state.is_active = false;
            state.candidates.clear();
        } else {
            self.update_candidates(state);
        }

        IMReturnValue::Consume
    }

    /// Handle return key - commit raw pinyin (English)
    fn handle_return(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.preedit.is_empty() {
            return IMReturnValue::Forward;
        }

        // Always commit raw pinyin (English) on Enter
        state.commit_text = state.preedit.clone();
        state.context.push_str(&state.preedit);
        state.preedit.clear();
        state.candidates.clear();
        state.is_active = false;

        IMReturnValue::Consume
    }

    /// Handle escape key
    fn handle_escape(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active {
            return IMReturnValue::Forward;
        }

        state.preedit.clear();
        state.candidates.clear();
        state.is_active = false;

        IMReturnValue::Consume
    }

    /// Handle space key (commit selected candidate)
    fn handle_space(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.preedit.is_empty() {
            return IMReturnValue::Forward;
        }

        // Commit the currently selected candidate
        let page_start = state.current_page * self.config.input.page_size;
        let candidate_idx = page_start + state.selected_index;

        if let Some(candidate) = state.candidates.get(candidate_idx).cloned() {
            self.commit_candidate(state, &candidate);
        } else if let Some(candidate) = state.candidates.first().cloned() {
            // Fallback to first candidate
            self.commit_candidate(state, &candidate);
        }

        IMReturnValue::Consume
    }

    /// Handle page up (for - key)
    fn handle_page_up(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.candidates.is_empty() {
            return IMReturnValue::Forward;
        }

        if state.current_page > 0 {
            state.current_page -= 1;
            state.selected_index = 0; // Reset selection when changing page
        }

        IMReturnValue::Consume
    }

    /// Handle page down (for + and = keys)
    fn handle_page_down(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.candidates.is_empty() {
            return IMReturnValue::Forward;
        }

        let max_page = (state.candidates.len() - 1) / self.config.input.page_size;
        if state.current_page < max_page {
            state.current_page += 1;
            state.selected_index = 0; // Reset selection when changing page
        }

        IMReturnValue::Consume
    }

    /// Handle left arrow key - select previous candidate
    fn handle_left(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.candidates.is_empty() {
            return IMReturnValue::Forward;
        }

        if state.selected_index > 0 {
            state.selected_index -= 1;
        } else {
            // At the beginning of current page, go to previous page's last item
            if state.current_page > 0 {
                state.current_page -= 1;
                state.selected_index = self.get_current_page_size(state) - 1;
            }
        }

        IMReturnValue::Consume
    }

    /// Handle right arrow key - select next candidate
    fn handle_right(&self, state: &mut InputState) -> IMReturnValue {
        if !state.is_active || state.candidates.is_empty() {
            return IMReturnValue::Forward;
        }

        let page_size = self.get_current_page_size(state);
        if state.selected_index < page_size - 1 {
            // Check if there's a candidate at the next position
            let page_start = state.current_page * self.config.input.page_size;
            let next_idx = page_start + state.selected_index + 1;
            if next_idx < state.candidates.len() {
                state.selected_index += 1;
            }
        } else {
            // At the end of current page, go to next page's first item
            let max_page = (state.candidates.len() - 1) / self.config.input.page_size;
            if state.current_page < max_page {
                state.current_page += 1;
                state.selected_index = 0;
            }
        }

        IMReturnValue::Consume
    }

    /// Get actual page size for current page (may be less on last page)
    fn get_current_page_size(&self, state: &InputState) -> usize {
        let page_start = state.current_page * self.config.input.page_size;
        let remaining = state.candidates.len().saturating_sub(page_start);
        remaining.min(self.config.input.page_size)
    }

    /// Update candidate list based on current preedit
    fn update_candidates(&self, state: &mut InputState) {
        state.candidates.clear();
        state.current_page = 0;
        state.selected_index = 0;

        if state.preedit.is_empty() {
            return;
        }

        // Parse pinyin into syllables
        let syllables = self.pinyin_parser.parse(&state.preedit);

        // Check if this is initial letter input (all single letters)
        let is_initial_input = syllables
            .iter()
            .all(|s| s.len() == 1 && s.chars().next().map_or(false, |c| c.is_ascii_alphabetic()));

        if is_initial_input && syllables.len() > 1 {
            // Handle initial letter input (e.g., "cs" -> "ce shi" -> "测试")
            self.add_initial_candidates(state, &syllables);
        } else if syllables.len() == 1 {
            // Single syllable - get direct matches from dictionary
            self.add_dictionary_candidates(state, &syllables[0]);

            // If no dictionary results, use built-in pinyin map
            if state.candidates.is_empty() {
                let pinyin_chars = get_candidates(&syllables[0]);
                for (idx, &ch) in pinyin_chars.iter().enumerate() {
                    state.candidates.push(Candidate {
                        text: ch.to_string(),
                        pinyin: syllables[0].clone(),
                        score: (100 - idx * 10) as f32,
                        source: PredictionSource::Dictionary,
                    });
                }
            }

            // Add fuzzy matches if enabled
            if self.config.input.fuzzy_pinyin {
                self.add_fuzzy_candidates(state, &syllables[0]);
            }
        } else {
            // Multiple syllables - try phrase matching
            self.add_phrase_candidates(state, &syllables);
        }

        // Add AI predictions if enabled and input is long enough
        if self.config.input.enable_phrase_prediction
            && state.preedit.len() >= self.config.input.min_ai_input_length
        {
            self.add_ai_candidates(state);
        }

        // Sort by score
        state.candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit candidates
        state.candidates.truncate(self.config.input.num_candidates);
    }

    /// Add candidates for initial letter input (e.g., "cs" -> "测试")
    fn add_initial_candidates(&self, state: &mut InputState, initials: &[String]) {
        let initial_map = get_initial_map();

        // Expand each initial to common pinyin (limit to top 3 per initial)
        let mut pinyin_combinations: Vec<Vec<String>> = vec![Vec::new()];

        for initial_str in initials {
            if let Some(ch) = initial_str.chars().next() {
                if let Some(pinyins) = initial_map.get(&ch) {
                    // Take top 3 most common pinyin for this initial
                    let top_pinyins: Vec<String> =
                        pinyins.iter().map(|s| s.to_string()).take(3).collect();

                    // Generate new combinations
                    let mut new_combinations: Vec<Vec<String>> = Vec::new();
                    for existing in &pinyin_combinations {
                        for py in &top_pinyins {
                            let mut new_comb: Vec<String> = existing.clone();
                            new_comb.push(py.clone());
                            new_combinations.push(new_comb);
                        }
                    }
                    pinyin_combinations = new_combinations;
                }
            }
        }

        // Limit combinations and try to find matches in dictionary
        let max_combinations = 20;
        for pinyin_combo in pinyin_combinations.into_iter().take(max_combinations) {
            let pinyin_str: String = pinyin_combo.join("");

            // Try to find this phrase in dictionary
            let entries = self.dictionary.lookup(&pinyin_str);
            let has_entries = !entries.is_empty();
            for entry in &entries {
                state.candidates.push(Candidate {
                    text: entry.word.clone(),
                    pinyin: entry.pinyin.clone(),
                    score: entry.frequency as f32 / 100.0 + 0.5, // Boost score for exact match
                    source: PredictionSource::Dictionary,
                });
            }

            // If no direct match, try individual character lookup
            if !has_entries && pinyin_combo.len() == 2 {
                let entries1 = self.dictionary.lookup(&pinyin_combo[0]);
                let entries2 = self.dictionary.lookup(&pinyin_combo[1]);

                // Generate a few combinations (limit to avoid too many)
                let max_per_char = 3;
                for (i, e1) in entries1.iter().take(max_per_char).enumerate() {
                    for (j, e2) in entries2.iter().take(max_per_char).enumerate() {
                        state.candidates.push(Candidate {
                            text: format!("{}{}", e1.word, e2.word),
                            pinyin: format!("{} {}", e1.pinyin, e2.pinyin),
                            score: (e1.frequency + e2.frequency) as f32 / 200.0
                                - (i + j) as f32 * 0.1,
                            source: PredictionSource::Dictionary,
                        });
                    }
                }
            }
        }
    }

    /// Add candidates from dictionary
    fn add_dictionary_candidates(&self, state: &mut InputState, pinyin: &str) {
        let entries = self.dictionary.lookup(pinyin);

        for entry in entries {
            state.candidates.push(Candidate {
                text: entry.word,
                pinyin: entry.pinyin,
                score: entry.frequency as f32 / 100.0,
                source: PredictionSource::Dictionary,
            });
        }
    }

    /// Add fuzzy pinyin matches
    fn add_fuzzy_candidates(&self, state: &mut InputState, pinyin: &str) {
        let fuzzy_matches = self.fuzzy_matcher.get_fuzzy_matches(pinyin);

        for fuzzy_pinyin in fuzzy_matches {
            if fuzzy_pinyin == pinyin {
                continue;
            }

            let entries = self.dictionary.lookup(&fuzzy_pinyin);
            for entry in entries {
                // Check if already in candidates
                if state.candidates.iter().any(|c| c.text == entry.word) {
                    continue;
                }

                state.candidates.push(Candidate {
                    text: entry.word,
                    pinyin: entry.pinyin,
                    score: (entry.frequency as f32 / 100.0) * 0.1, // Significantly lower score for fuzzy matches
                    source: PredictionSource::FuzzyMatch,
                });
            }
        }
    }

    /// Add phrase candidates for multi-syllable input
    fn add_phrase_candidates(&self, state: &mut InputState, syllables: &[String]) {
        // Simple phrase matching: combine single character candidates
        let pinyin_str = syllables.join("");
        let pinyin_with_spaces = syllables.join(" ");

        // Priority 1: Complete phrases from dictionary (highest priority)
        // First try with spaces (user/system dictionary format)
        let entries = self.dictionary.lookup(&pinyin_with_spaces);
        log::debug!(
            "Dictionary lookup '{}' found {} entries",
            pinyin_with_spaces,
            entries.len()
        );
        for entry in &entries {
            // Check if already in candidates from combinations
            if let Some(existing) = state.candidates.iter_mut().find(|c| c.text == entry.word) {
                // Update score to highest priority
                existing.score = 100.0 + entry.frequency as f32 / 100.0;
                existing.source = PredictionSource::Dictionary;
            } else {
                state.candidates.push(Candidate {
                    text: entry.word.clone(),
                    pinyin: entry.pinyin.clone(),
                    score: 100.0 + entry.frequency as f32 / 100.0,
                    source: PredictionSource::Dictionary,
                });
            }
        }

        // Then try without spaces (for compatibility)
        let entries_no_space = self.dictionary.lookup(&pinyin_str);
        log::debug!(
            "Dictionary lookup '{}' found {} entries",
            pinyin_str,
            entries_no_space.len()
        );
        for entry in entries_no_space {
            // Check if already in candidates
            if let Some(existing) = state.candidates.iter_mut().find(|c| c.text == entry.word) {
                let dict_score = 100.0 + entry.frequency as f32 / 100.0;
                if dict_score > existing.score {
                    existing.score = dict_score;
                    existing.source = PredictionSource::Dictionary;
                }
            } else {
                state.candidates.push(Candidate {
                    text: entry.word,
                    pinyin: entry.pinyin,
                    score: 100.0 + entry.frequency as f32 / 100.0,
                    source: PredictionSource::Dictionary,
                });
            }
        }

        // Priority 2: Generate word combinations (medium priority, limited quantity)
        if syllables.len() >= 2 && syllables.len() <= 4 {
            self.generate_combinations(state, syllables);
        }

        // Priority 3: Single characters by syllable (in syllable order, limited quantity)
        // Each syllable gets decreasing priority: first syllable = higher, second = lower, etc.
        // This allows users to select characters one by one to build words
        for (syllable_idx, syl) in syllables.iter().enumerate() {
            let entries = self.dictionary.lookup(syl);
            log::debug!(
                "Dictionary lookup for single syllable '{}' found {} entries",
                syl,
                entries.len()
            );

            // Only take top 5 characters per syllable to avoid too many candidates
            for (char_idx, entry) in entries.iter().take(5).enumerate() {
                // Check if already in candidates (avoid duplicates)
                if !state.candidates.iter().any(|c| c.text == entry.word) {
                    // Decreasing priority for each subsequent syllable
                    // First syllable: 30.0, second: 20.0, third: 10.0, fourth: 0.0
                    let syllable_priority = 30.0 - (syllable_idx as f32 * 10.0);
                    // Decreasing priority for characters within same syllable
                    let char_priority = (5.0 - char_idx as f32) * 0.5;
                    state.candidates.push(Candidate {
                        text: entry.word.clone(),
                        pinyin: entry.pinyin.clone(),
                        score: syllable_priority + char_priority + entry.frequency as f32 / 200.0,
                        source: PredictionSource::Dictionary,
                    });
                }
            }
        }
    }

    /// Generate character combinations for multiple syllables
    fn generate_combinations(&self, state: &mut InputState, syllables: &[String]) {
        // Get candidates for each syllable from dictionary (sorted by frequency)
        let all_candidates: Vec<Vec<(String, u64)>> = syllables
            .iter()
            .map(|s| {
                let entries = self.dictionary.lookup(s);
                // If dictionary lookup returns empty, try built-in pinyin map
                let entries = if entries.is_empty() {
                    let built_in = get_candidates(s);
                    built_in
                        .iter()
                        .take(10)
                        .enumerate()
                        .map(|(idx, &ch)| (ch.to_string(), (100 - idx as u64 * 10).max(1)))
                        .collect::<Vec<_>>()
                } else {
                    // Take top candidates with their frequencies
                    entries
                        .iter()
                        .take(10)
                        .map(|e| (e.word.clone(), e.frequency))
                        .collect::<Vec<_>>()
                };
                entries
            })
            .collect();

        // Log for debugging
        log::debug!(
            "generate_combinations: syllables={:?}, candidates_per_syllable={:?}",
            syllables,
            all_candidates.iter().map(|c| c.len()).collect::<Vec<_>>()
        );

        // Generate combinations with frequency-based scoring
        let mut combinations = Vec::new();
        let max_per_syllable = 2;

        // Simple cartesian product for small number of syllables
        if syllables.len() == 2 {
            // Only generate if both syllables have candidates
            if !all_candidates[0].is_empty() && !all_candidates[1].is_empty() {
                for i in 0..all_candidates[0].len().min(max_per_syllable) {
                    for j in 0..all_candidates[1].len().min(max_per_syllable) {
                        let (word1, freq1) = &all_candidates[0][i];
                        let (word2, freq2) = &all_candidates[1][j];
                        let text = format!("{}{}", word1, word2);
                        // Score based on combined frequency (normalized)
                        let freq_score = (freq1 + freq2) as f32 / 1000.0;
                        let pos_penalty = (i + j) as f32 * 0.05;
                        combinations.push((text, freq_score - pos_penalty));
                    }
                }
            }
        } else if syllables.len() == 3 {
            if all_candidates.iter().all(|c| !c.is_empty()) {
                for i in 0..all_candidates[0].len().min(max_per_syllable) {
                    for j in 0..all_candidates[1].len().min(max_per_syllable) {
                        for k in 0..all_candidates[2].len().min(max_per_syllable) {
                            let (word1, freq1) = &all_candidates[0][i];
                            let (word2, freq2) = &all_candidates[1][j];
                            let (word3, freq3) = &all_candidates[2][k];
                            let text = format!("{}{}{}", word1, word2, word3);
                            let freq_score = (freq1 + freq2 + freq3) as f32 / 1000.0;
                            let pos_penalty = (i + j + k) as f32 * 0.04;
                            combinations.push((text, freq_score - pos_penalty));
                        }
                    }
                }
            }
        } else if syllables.len() == 4 {
            if all_candidates.iter().all(|c| !c.is_empty()) {
                // Only generate 4 combinations (2×2×2×2=4) to avoid too many
                for i in 0..all_candidates[0].len().min(2) {
                    for j in 0..all_candidates[1].len().min(2) {
                        for k in 0..all_candidates[2].len().min(2) {
                            for l in 0..all_candidates[3].len().min(2) {
                                let (word1, freq1) = &all_candidates[0][i];
                                let (word2, freq2) = &all_candidates[1][j];
                                let (word3, freq3) = &all_candidates[2][k];
                                let (word4, freq4) = &all_candidates[3][l];
                                let text = format!("{}{}{}{}", word1, word2, word3, word4);
                                let freq_score = (freq1 + freq2 + freq3 + freq4) as f32 / 1000.0;
                                let pos_penalty = (i + j + k + l) as f32 * 0.03;
                                combinations.push((text, freq_score - pos_penalty));
                            }
                        }
                    }
                }
            }
        }

        // Add combinations as candidates (skip if already exists from dictionary)
        for (text, score) in combinations.into_iter().take(10) {
            if !state.candidates.iter().any(|c| c.text == text) {
                state.candidates.push(Candidate {
                    text,
                    pinyin: syllables.join(""),
                    score: 50.0 + score, // Medium priority: higher than single chars, lower than complete phrases
                    source: PredictionSource::Dictionary,
                });
            }
        }
    }

    /// Add AI-predicted candidates
    fn add_ai_candidates(&self, state: &mut InputState) {
        // Skip if model is disabled or not available
        if !self.model.is_available() {
            return;
        }

        // Skip if phrase prediction is disabled in config
        if !self.config.input.enable_phrase_prediction {
            return;
        }

        // Skip if input is too short for AI prediction
        if state.preedit.len() < self.config.input.min_ai_input_length {
            return;
        }

        let predictions = self.model.predict(&state.context, &state.preedit);

        for pred in predictions {
            // Check if already in candidates
            if state.candidates.iter().any(|c| c.text == pred.text) {
                continue;
            }

            state.candidates.push(Candidate {
                text: pred.text,
                pinyin: state.preedit.clone(),
                score: pred.confidence * 0.9, // AI predictions get high score
                source: pred.source,
            });
        }
    }

    /// Commit a candidate
    fn commit_candidate(&self, state: &mut InputState, candidate: &Candidate) {
        // Update dictionary frequency
        self.dictionary
            .update_frequency(&candidate.pinyin, &candidate.text);

        // Set commit text for C++ to retrieve
        state.commit_text = candidate.text.clone();

        // Update context
        state.context.push_str(&candidate.text);

        // Limit context length by characters (not bytes)
        const MAX_CONTEXT_CHARS: usize = 100;
        let current_chars = state.context.chars().count();
        if current_chars > MAX_CONTEXT_CHARS {
            // Truncate to MAX_CONTEXT_CHARS safely
            let truncated: String = state.context.chars().take(MAX_CONTEXT_CHARS).collect();
            state.context = truncated;
        }

        // Clear preedit
        state.preedit.clear();
        state.candidates.clear();
        state.is_active = false;

        // Save user dictionary periodically (every 5 commits)
        let count = self.commit_counter.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= 5 {
            self.commit_counter.store(0, Ordering::SeqCst);
            if let Err(e) = self.dictionary.save_user_dictionary() {
                log::warn!("Failed to save user dictionary: {}", e);
            }
        }
    }

    /// Get current preedit text
    pub fn get_preedit(&self, ic: *mut FcitxInputContext) -> String {
        let state = self.get_state(ic);
        state.preedit.clone()
    }

    /// Get all candidates
    pub fn get_candidates(&self, ic: *mut FcitxInputContext) -> Vec<Candidate> {
        let state = self.get_state(ic);
        state.candidates.clone()
    }

    /// Get current page index
    pub fn get_current_page(&self, ic: *mut FcitxInputContext) -> usize {
        let state = self.get_state(ic);
        state.current_page
    }

    /// Get selected candidate index within current page (relative index 0 to page_size-1)
    pub fn get_selected_index(&self, ic: *mut FcitxInputContext) -> usize {
        let state = self.get_state(ic);
        // selected_index is already page-relative (0 to page_size-1)
        state.selected_index
    }

    /// Get total number of candidates
    pub fn get_total_candidates(&self, ic: *mut FcitxInputContext) -> usize {
        let state = self.get_state(ic);
        state.candidates.len()
    }

    /// Get page size from config
    pub fn get_config_page_size(&self) -> usize {
        self.config.input.page_size
    }

    /// Get commit text and clear it (for C++ to retrieve and actually commit)
    pub fn get_commit_text(&self, ic: *mut FcitxInputContext) -> String {
        let mut state = self.get_state(ic);
        let text = state.commit_text.clone();
        // Clear commit text after retrieving
        if !text.is_empty() {
            state.commit_text.clear();
            self.update_state(ic, state);
        }
        text
    }

    /// Reset input state
    pub fn reset(&self, ic: *mut FcitxInputContext) {
        let state = InputState::default();
        self.update_state(ic, state);
    }

    /// Focus in handler
    pub fn focus_in(&self, ic: *mut FcitxInputContext) {
        // Restore or create state
        let _ = self.get_state(ic);
    }

    /// Focus out handler
    pub fn focus_out(&self, ic: *mut FcitxInputContext) {
        // Save state and clean up
        let mut state = self.get_state(ic);
        state.preedit.clear();
        state.candidates.clear();
        state.is_active = false;
        self.update_state(ic, state);
    }

    /// Check if the AI model is available
    pub fn is_model_available(&self) -> bool {
        self.model.is_available()
    }

    /// Get the configured page size
    pub fn get_page_size(&self) -> usize {
        self.config.input.page_size
    }
}

use std::collections::HashMap;

impl Default for InputEngine {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = InputEngine::default();
        assert!(engine.model.is_available());
    }

    #[test]
    fn test_letter_input() {
        let engine = InputEngine::default();
        let ic = std::ptr::null_mut();

        let result = engine.handle_key(
            std::ptr::null_mut(),
            ic,
            0x006e, // 'n'
            0,
            0,
            false,
        );

        assert_eq!(result, IMReturnValue::Consume);
        assert_eq!(engine.get_preedit(ic), "n");
    }
}

use log::{error, info};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

type Item = (usize, usize, usize, usize);

pub struct Fuzzer {
    pub tests_count: usize,
    pub min_len: usize,
    pub max_len: usize,
    pub rng: ThreadRng,
}

impl Fuzzer {
    pub fn new(tests_count: usize, min_len: usize, max_len: usize) -> Self {
        Self {
            tests_count,
            min_len,
            max_len,
            rng: rand::rng(),
        }
    }

    pub fn fuzz_equivalence(
        &mut self,
        before_path: &str,
        after1_path: &str,
        after2_path: &str,
    ) -> Result<(), String> {
        let before_text = self.read_text(before_path)?;
        let after1_text = self.read_text(after1_path)?;
        let after2_text = self.read_text(after2_path)?;

        let (before_start, before_prods, before_alpha) = self.parse_before(&before_text)?;
        let (after1_start, after1_prods, after1_alpha) = self.parse_after(&after1_text)?;
        let (after2_start, after2_prods, after2_alpha) = self.parse_after(&after2_text)?;

        let alphabet = self.merge_alphabet(&[before_alpha, after1_alpha, after2_alpha])?;
        info!(
            "Alphabet: {:?}",
            alphabet.iter().map(|b| *b as char).collect::<Vec<_>>()
        );

        let in_words = self.generate_in_words(&before_start, &before_prods)?;
        let out_words = self.generate_out_words(&in_words)?;

        let total = 2 * self.tests_count;
        let mut passed = 0usize;

        info!(
            "Testing equivalence on {} words ({} IN + {} OUT)...",
            total, self.tests_count, self.tests_count
        );

        for (test_index, word) in in_words.iter().chain(out_words.iter()).enumerate() {
            let before_accepts = self.contains(before_start, &before_prods, word);
            let after1_accepts = self.contains(after1_start, &after1_prods, word);
            let after2_accepts = self.contains(after2_start, &after2_prods, word);

            if before_accepts == after1_accepts && before_accepts == after2_accepts {
                passed += 1;
                let done = test_index + 1;
                let in_out_type = if before_accepts {
                    "IN"
                } else {
                    "OUT"
                };
                if done % 10 == 0 || done == total {
                    info!(
                        "passed {}/{} (last='{}' type='{}')",
                        done,
                        total,
                        String::from_utf8_lossy(word),
                        in_out_type
                    );
                }
            } else {
                error!(
                    "Mismatch at {}/{}: word='{}' | before={} after1={} after2={}",
                    test_index + 1,
                    total,
                    String::from_utf8_lossy(word),
                    before_accepts,
                    after1_accepts,
                    after2_accepts
                );
                return Err("Mismatch".into());
            }
        }

        info!("All tests passed: {}/{}", passed, total);
        Ok(())

    }

    fn generate_in_words(
        &mut self,
        start_nonterminal: &usize,
        productions: &Vec<Vec<Vec<i32>>>,
    ) -> Result<Vec<Vec<u8>>, String> {
        let mut words = Vec::with_capacity(self.tests_count);
        let mut seen = HashSet::<Vec<u8>>::new();

        let mut attempts = 0usize;
        while words.len() < self.tests_count {
            attempts += 1;
            if attempts > 300_000 {
                return Err(format!(
                    "IN words generation failed: got {} / {}",
                    words.len(),
                    self.tests_count
                ));
            }

            let mut sentential_form = VecDeque::<i32>::new();
            sentential_form.push_back(*start_nonterminal as i32);

            let mut steps = 0usize;
            let mut maybe_word: Option<Vec<u8>> = None;

            while steps < 50_000 {
                steps += 1;

                if sentential_form.iter().all(|&sym| sym < 0) {
                    let mut word = Vec::<u8>::with_capacity(sentential_form.len());
                    for &sym in sentential_form.iter() {
                        word.push((-sym - 1) as u8);
                    }
                    maybe_word = Some(word);
                    break;
                }

                if sentential_form.len() > 4 * self.max_len {
                    break;
                }

                let nonterminal_positions: Vec<usize> = sentential_form
                    .iter()
                    .enumerate()
                    .filter_map(|(pos, &sym)| (sym >= 0).then_some(pos))
                    .collect();

                if nonterminal_positions.is_empty() {
                    break;
                }

                let chosen_position =
                    nonterminal_positions[self.rng.random_range(0..nonterminal_positions.len())];
                let chosen_nonterminal = sentential_form[chosen_position] as usize;

                let alternatives = &productions[chosen_nonterminal];
                if alternatives.is_empty() {
                    break;
                }

                let chosen_rhs = &alternatives[self.rng.random_range(0..alternatives.len())];

                sentential_form.remove(chosen_position);
                for (offset, sym) in chosen_rhs.iter().cloned().enumerate() {
                    sentential_form.insert(chosen_position + offset, sym);
                }
            }

            let Some(word) = maybe_word else { continue; };

            if word.len() < self.min_len || word.len() > self.max_len {
                continue;
            }

            if self.contains(*start_nonterminal, productions, &word) && seen.insert(word.clone()) {
                words.push(word);
            }
        }

        Ok(words)
    }

    fn generate_out_words(&self, in_words: &Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>, String> {
        let mut out_words = Vec::with_capacity(in_words.len());
        for word in in_words {
            let mut out = Vec::with_capacity(word.len() + 3);
            out.extend_from_slice(b"cab");
            out.extend_from_slice(word);
            out_words.push(out);
        }
        Ok(out_words)
    }

    fn parse_before(&mut self, text: &str) -> Result<(usize, Vec<Vec<Vec<i32>>>, Vec<u8>), String> {
        self.parse_cfg(text, false)
    }

    fn parse_after(&mut self, text: &str) -> Result<(usize, Vec<Vec<Vec<i32>>>, Vec<u8>), String> {
        self.parse_cfg(text, true)
    }

    fn parse_cfg(
        &mut self,
        text: &str,
        one_alt_per_line: bool,
    ) -> Result<(usize, Vec<Vec<Vec<i32>>>, Vec<u8>), String> {
        let mut rules: Vec<(String, Vec<Vec<String>>)> = Vec::new();
        let mut start_name: Option<String> = None;

        for (line_index, raw_line) in text.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split("->").collect();
            if parts.len() != 2 {
                return Err(format!("Line {}: expected 'LHS -> RHS': {}", line_index + 1, line));
            }

            let lhs = parts[0].trim().to_string();
            start_name.get_or_insert(lhs.clone());

            let rhs_text = parts[1].trim();
            let mut alts = Vec::<Vec<String>>::new();

            if one_alt_per_line {
                alts.push(self.tokenize(rhs_text));
            } else {
                for alt in rhs_text.split('|') {
                    alts.push(self.tokenize(alt.trim()));
                }
            }

            rules.push((lhs, alts));
        }

        let start_name = start_name.ok_or("Empty grammar")?;

        let is_nonterminal = |token: &str| -> bool {
            if token == "eps" || token == "ε" {
                return false;
            }
            if token.starts_with('<') && token.ends_with('>') {
                return true;
            }
            token
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false)
        };

        let mut nonterminal_ids = HashMap::<String, usize>::new();
        let mut nonterminal_names = Vec::<String>::new();

        let get_or_create_nonterminal_id =
            |name: &str, ids: &mut HashMap<String, usize>, names: &mut Vec<String>| -> usize {
                if let Some(&id) = ids.get(name) {
                    id
                } else {
                    let id = names.len();
                    ids.insert(name.to_string(), id);
                    names.push(name.to_string());
                    id
                }
            };

        for (lhs, _) in rules.iter() {
            get_or_create_nonterminal_id(lhs, &mut nonterminal_ids, &mut nonterminal_names);
        }
        for (_, alts) in rules.iter() {
            for rhs in alts {
                for token in rhs {
                    if is_nonterminal(token) {
                        get_or_create_nonterminal_id(
                            token,
                            &mut nonterminal_ids,
                            &mut nonterminal_names,
                        );
                    }
                }
            }
        }

        let start = *nonterminal_ids.get(&start_name).unwrap();
        let mut productions: Vec<Vec<Vec<i32>>> = vec![Vec::new(); nonterminal_names.len()];
        let mut alphabet_set = HashSet::<u8>::new();

        for (lhs, alts) in rules {
            let lhs_id = *nonterminal_ids.get(&lhs).unwrap();

            for rhs_tokens in alts {
                if rhs_tokens.len() == 1 && (rhs_tokens[0] == "eps" || rhs_tokens[0] == "ε") {
                    productions[lhs_id].push(vec![]);
                    continue;
                }

                let mut encoded_rhs = Vec::<i32>::new();
                for token in rhs_tokens {
                    if token == "eps" || token == "ε" {
                        continue;
                    }

                    if is_nonterminal(&token) {
                        encoded_rhs.push(*nonterminal_ids.get(&token).unwrap() as i32);
                    } else {
                        let bytes = token.as_bytes();
                        if bytes.len() != 1 {
                            return Err(format!("Terminal must be single char. Got: {}", token));
                        }
                        alphabet_set.insert(bytes[0]);
                        encoded_rhs.push(-((bytes[0] as i32) + 1));
                    }
                }

                productions[lhs_id].push(encoded_rhs);
            }
        }

        let mut alphabet: Vec<u8> = alphabet_set.into_iter().collect();
        alphabet.sort();

        info!(
            "Parsed grammar start='{}', nonterms={}, terminals={}",
            start_name,
            productions.len(),
            alphabet.len()
        );

        Ok((start, productions, alphabet))
    }

    fn tokenize(&self, rhs: &str) -> Vec<String> {
        let trimmed = rhs.trim();
        if trimmed.is_empty() {
            return vec![];
        }
        if trimmed == "eps" || trimmed == "ε" {
            return vec!["eps".to_string()];
        }
        trimmed.split_whitespace().map(|t| t.to_string()).collect()
    }

    fn contains(&mut self, start: usize, productions: &Vec<Vec<Vec<i32>>>, word: &Vec<u8>) -> bool {
        let word_len = word.len();
        let mut chart: Vec<HashSet<Item>> = (0..=word_len).map(|_| HashSet::new()).collect();
        let mut agenda: Vec<VecDeque<Item>> = (0..=word_len).map(|_| VecDeque::new()).collect();

        for production_index in 0..productions[start].len() {
            let start_item = (start, production_index, 0, 0);
            chart[0].insert(start_item);
            agenda[0].push_back(start_item);
        }

        for position in 0..=word_len {
            while let Some((lhs_nt, prod_idx, dot_idx, origin)) = agenda[position].pop_front() {
                let rhs = &productions[lhs_nt][prod_idx];

                if dot_idx >= rhs.len() {
                    let parent_items: Vec<Item> = chart[origin].iter().cloned().collect();

                    for (parent_lhs, parent_prod, parent_dot, parent_origin) in parent_items {
                        let parent_rhs = &productions[parent_lhs][parent_prod];
                        if parent_dot < parent_rhs.len() {
                            let expected = parent_rhs[parent_dot];
                            if expected >= 0 && expected as usize == lhs_nt {
                                let next_item = (parent_lhs, parent_prod, parent_dot + 1, parent_origin);
                                if chart[position].insert(next_item) {
                                    agenda[position].push_back(next_item);
                                }
                            }
                        }
                    }
                    continue;
                }

                let expected = rhs[dot_idx];

                if expected >= 0 {
                    let predicted_nt = expected as usize;
                    for predicted_prod in 0..productions[predicted_nt].len() {
                        let next_item = (predicted_nt, predicted_prod, 0, position);
                        if chart[position].insert(next_item) {
                            agenda[position].push_back(next_item);
                        }
                    }
                } else if position < word_len {
                    let expected_terminal = (-expected - 1) as u8;
                    if word[position] == expected_terminal {
                        let next_item = (lhs_nt, prod_idx, dot_idx + 1, origin);
                        if chart[position + 1].insert(next_item) {
                            agenda[position + 1].push_back(next_item);
                        }
                    }
                }
            }
        }

        for production_index in 0..productions[start].len() {
            let rhs_len = productions[start][production_index].len();
            if chart[word_len].contains(&(start, production_index, rhs_len, 0)) {
                return true;
            }
        }

        false
    }

    fn read_text(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|err| format!("read {}: {}", path, err))
    }

    fn merge_alphabet(&self, lists: &[Vec<u8>]) -> Result<Vec<u8>, String> {
        let mut set = HashSet::<u8>::new();
        for list in lists {
            for &terminal in list {
                set.insert(terminal);
            }
        }
        let mut alphabet: Vec<u8> = set.into_iter().collect();
        alphabet.sort();
        if alphabet.is_empty() {
            Err("Alphabet is empty (no terminals found).".to_string())
        } else {
            Ok(alphabet)
        }
    }
}

pub fn start_fuzzer(before_path: &str, after1_path: &str, after2_path: &str) {
    let mut fuzzer = Fuzzer::new(50, 1, 40);
    match fuzzer.fuzz_equivalence(before_path, after1_path, after2_path) {
        Ok(()) => info!("OK: no mismatches found."),
        Err(err) => error!("Fuzzer failed: {}", err),
    }
}

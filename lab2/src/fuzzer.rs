use log::{error, info};
use rand::Rng;
use rand::rngs::ThreadRng;
use regex::Regex;
use std::collections::HashSet;

struct Fuzzer {
    tests_count: usize,
    min_str_len: usize,
    max_str_len: usize,
    alphabet: Vec<char>,
    start_node_dfa: usize,
    final_nodes_dfa: Vec<usize>,
    start_node_nfa: usize,
    final_nodes_nfa: Vec<usize>,
    start_node_afa: usize,
    final_nodes_afa: Vec<usize>,
    dfa: Vec<(usize, usize, usize)>,
    nfa: Vec<(usize, usize, usize)>,
    afa: Vec<(usize, usize, usize)>,
    re: Regex,
    ext_re: Regex,
    rng: ThreadRng,
}

impl Fuzzer {
    fn new() -> Self {
        Self {
            tests_count: 1000,
            min_str_len: 10,
            max_str_len: 100,
            start_node_dfa: 42,
            alphabet: vec!['a', 'b', 'c'],
            final_nodes_dfa: vec![0, 1, 2, 3, 4, 5],
            start_node_nfa: 0,
            final_nodes_nfa: vec![20, 21],
            start_node_afa: 0,
            final_nodes_afa: vec![4, 5, 6, 7, 8, 9],
            dfa: vec![
                (0, 33, 0),
                (0, 58, 1),
                (0, 59, 2),
                (1, 60, 0),
                (1, 60, 1),
                (1, 60, 2),
                (2, 32, 0),
                (2, 0, 1),
                (2, 60, 2),
                (3, 60, 0),
                (3, 1, 1),
                (3, 54, 2),
                (4, 60, 0),
                (4, 1, 1),
                (4, 56, 2),
                (5, 60, 0),
                (5, 1, 1),
                (5, 60, 2),
                (6, 19, 0),
                (6, 38, 1),
                (6, 61, 2),
                (7, 33, 0),
                (7, 43, 1),
                (7, 61, 2),
                (8, 33, 0),
                (8, 58, 1),
                (8, 61, 2),
                (9, 33, 0),
                (9, 58, 1),
                (9, 62, 2),
                (10, 15, 0),
                (10, 6, 1),
                (10, 52, 2),
                (11, 13, 0),
                (11, 6, 1),
                (11, 52, 2),
                (12, 14, 0),
                (12, 6, 1),
                (12, 52, 2),
                (13, 10, 0),
                (13, 8, 1),
                (13, 59, 2),
                (14, 11, 0),
                (14, 8, 1),
                (14, 59, 2),
                (15, 12, 0),
                (15, 7, 1),
                (15, 59, 2),
                (16, 18, 0),
                (16, 8, 1),
                (16, 59, 2),
                (17, 16, 0),
                (17, 8, 1),
                (17, 59, 2),
                (18, 17, 0),
                (18, 7, 1),
                (18, 59, 2),
                (19, 31, 0),
                (19, 9, 1),
                (19, 60, 2),
                (20, 32, 0),
                (20, 9, 1),
                (20, 60, 2),
                (21, 24, 0),
                (21, 9, 1),
                (21, 60, 2),
                (22, 25, 0),
                (22, 9, 1),
                (22, 60, 2),
                (23, 21, 0),
                (23, 26, 1),
                (23, 49, 2),
                (24, 15, 0),
                (24, 28, 1),
                (24, 52, 2),
                (25, 22, 0),
                (25, 28, 1),
                (25, 52, 2),
                (26, 19, 0),
                (26, 23, 1),
                (26, 55, 2),
                (27, 19, 0),
                (27, 23, 1),
                (27, 59, 2),
                (28, 19, 0),
                (28, 38, 1),
                (28, 55, 2),
                (29, 19, 0),
                (29, 38, 1),
                (29, 59, 2),
                (30, 19, 0),
                (30, 53, 1),
                (30, 59, 2),
                (31, 18, 0),
                (31, 58, 1),
                (31, 59, 2),
                (32, 20, 0),
                (32, 58, 1),
                (32, 59, 2),
                (33, 32, 0),
                (33, 56, 1),
                (33, 60, 2),
                (34, 25, 0),
                (34, 56, 1),
                (34, 60, 2),
                (35, 34, 0),
                (35, 28, 1),
                (35, 49, 2),
                (36, 34, 0),
                (36, 28, 1),
                (36, 52, 2),
                (37, 34, 0),
                (37, 28, 1),
                (37, 46, 2),
                (38, 45, 0),
                (38, 27, 1),
                (38, 50, 2),
                (39, 45, 0),
                (39, 29, 1),
                (39, 50, 2),
                (40, 45, 0),
                (40, 29, 1),
                (40, 51, 2),
                (41, 45, 0),
                (41, 29, 1),
                (41, 47, 2),
                (42, 45, 0),
                (42, 29, 1),
                (42, 48, 2),
                (43, 60, 0),
                (43, 30, 1),
                (43, 56, 2),
                (44, 60, 0),
                (44, 30, 1),
                (44, 60, 2),
                (45, 40, 0),
                (45, 60, 1),
                (45, 60, 2),
                (46, 60, 0),
                (46, 60, 1),
                (46, 35, 2),
                (47, 33, 0),
                (47, 58, 1),
                (47, 37, 2),
                (48, 60, 0),
                (48, 60, 1),
                (48, 36, 2),
                (49, 33, 0),
                (49, 58, 1),
                (49, 41, 2),
                (50, 33, 0),
                (50, 58, 1),
                (50, 42, 2),
                (51, 60, 0),
                (51, 60, 1),
                (51, 40, 2),
                (52, 60, 0),
                (52, 60, 1),
                (52, 39, 2),
                (53, 60, 0),
                (53, 44, 1),
                (53, 56, 2),
                (54, 33, 0),
                (54, 58, 1),
                (54, 55, 2),
                (55, 33, 0),
                (55, 58, 1),
                (55, 57, 2),
                (56, 33, 0),
                (56, 58, 1),
                (56, 59, 2),
                (57, 60, 0),
                (57, 60, 1),
                (57, 54, 2),
                (58, 60, 0),
                (58, 60, 1),
                (58, 56, 2),
                (59, 60, 0),
                (59, 60, 1),
                (59, 58, 2),
                (60, 60, 0),
                (60, 60, 1),
                (60, 60, 2),
                (61, 2, 0),
                (61, 4, 1),
                (61, 3, 2),
                (62, 5, 0),
                (62, 5, 1),
                (62, 4, 2),
            ],
            nfa: vec![
                (0, 1, 0),
                (0, 2, 1),
                (0, 5, 1),
                (0, 10, 1),
                (0, 11, 1),
                (0, 16, 1),
                (0, 3, 2),
                (1, 0, 0),
                (1, 4, 0),
                (2, 0, 1),
                (2, 4, 1),
                (3, 0, 2),
                (3, 4, 2),
                (4, 5, 1),
                (4, 10, 1),
                (4, 11, 1),
                (4, 16, 1),
                (5, 6, 0),
                (5, 12, 0),
                (5, 15, 0),
                (5, 17, 0),
                (5, 8, 1),
                (5, 13, 1),
                (5, 14, 2),
                (6, 7, 0),
                (7, 5, 0),
                (7, 10, 0),
                (7, 11, 0),
                (7, 16, 0),
                (8, 9, 1),
                (9, 5, 1),
                (9, 10, 1),
                (9, 11, 1),
                (9, 16, 1),
                (10, 12, 0),
                (10, 15, 0),
                (10, 17, 0),
                (10, 13, 1),
                (10, 14, 2),
                (11, 12, 0),
                (11, 15, 0),
                (11, 13, 1),
                (11, 14, 2),
                (12, 11, 1),
                (13, 11, 2),
                (14, 13, 2),
                (15, 10, 0),
                (15, 11, 0),
                (15, 16, 0),
                (16, 17, 0),
                (17, 18, 1),
                (18, 19, 2),
                (19, 20, 0),
                (19, 20, 1),
                (19, 20, 2),
                (19, 21, 0),
                (19, 21, 1),
                (20, 21, 1),
            ],
            afa: vec![
                (0, 1, 0),
                (0, 0, 1),
                (0, 0, 2),
                (1, 1, 0),
                (1, 2, 1),
                (1, 0, 2),
                (2, 1, 0),
                (2, 0, 1),
                (2, 3, 2),
                (3, 4, 0),
                (3, 5, 1),
                (3, 6, 2),
                (4, 1, 0),
                (4, 7, 1),
                (4, 0, 2),
                (5, 1, 0),
                (5, 8, 1),
                (5, 0, 2),
                (6, 1, 0),
                (6, 9, 1),
                (6, 0, 2),
                (7, 1, 0),
                (7, 0, 1),
                (7, 3, 2),
                (8, 1, 0),
                (8, 0, 1),
                (8, 0, 2),
                (9, 1, 0),
                (9, 0, 1),
                (9, 0, 2),
            ],
            re: Regex::new("^(aa|bb|cc)*b(aaa|bbb)*((ab|bc|ccc)*aa)*abc(a|b|c)(b|)$").unwrap(),
            ext_re: Regex::new("^(aa|bb|cc)*b(aaa|bbb)*((ab|bc|ccc)*aa)*abc[abc]b?$").unwrap(),
            rng: rand::rng(),
        }
    }

    fn nfa_check(&self, word: &String) -> bool {
        let mut queue: HashSet<usize> = HashSet::new();
        queue.insert(self.start_node_nfa);

        for char in word.chars() {
            let mut next_queue: HashSet<usize> = HashSet::new();
            for &state in &queue {
                for &(from, to, ch) in &self.nfa {
                    let new_ch = match Self::mapper(ch) {
                        Some(char) => char,
                        None => return false,
                    };
                    if from == state && char == new_ch {
                        next_queue.insert(to);
                    }
                }
            }

            queue = next_queue;
            if queue.is_empty() {
                break;
            }
        }

        queue.iter().any(|s| self.final_nodes_nfa.contains(s))
    }

    fn dfa_check(&self, word: &String) -> bool {
        let mut cur_state = self.start_node_dfa;
        for char in word.chars() {
            let mut found = false;
            for &(from, to, ch) in &self.dfa {
                let new_ch = match Self::mapper(ch) {
                    Some(char) => char,
                    None => return false,
                };
                if from == cur_state && char == new_ch {
                    cur_state = to;
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        self.final_nodes_dfa.contains(&cur_state)
    }

    fn afa_check(&self, word: &String) -> bool {
        let mut cur_state = self.start_node_afa;
        for char in word.chars() {
            let mut found = false;
            for &(from, to, ch) in &self.afa {
                let new_ch = match Self::mapper(ch) {
                    Some(char) => char,
                    None => return false,
                };
                if from == cur_state && char == new_ch {
                    cur_state = to;
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        self.final_nodes_afa.contains(&cur_state) && self.nfa_check(word)
    }
    fn get_rand_word(&mut self) -> String {
        let len = self.rng.random_range(self.min_str_len..self.max_str_len);
        let mut string = String::new();

        for _ in 0..len {
            string.push(self.alphabet[self.rng.random_range(0..self.alphabet.len())])
        }

        string
    }
    fn mapper(idx: usize) -> Option<char> {
        match idx {
            0 => Option::from('a'),
            1 => Option::from('b'),
            2 => Option::from('c'),
            _ => None,
        }
    }
}

pub fn start_fuzzer() {
    let mut new_fuzzer = Fuzzer::new();
    let mut test_passed: usize = 0;

    for _ in 0..new_fuzzer.tests_count {
        let new_string = new_fuzzer.get_rand_word();
        let res_dfa_check = new_fuzzer.dfa_check(&new_string);
        let res_nfa_check = new_fuzzer.nfa_check(&new_string);
        let res_afa_check = new_fuzzer.afa_check(&new_string);
        let res_re_match = new_fuzzer.re.is_match(&new_string);
        let res_ext_re_match = new_fuzzer.ext_re.is_match(&new_string);
        if res_re_match == res_dfa_check
            && res_re_match == res_nfa_check
            && res_re_match == res_ext_re_match
            && res_re_match == res_afa_check
        {
            test_passed += 1;
            info!(
                "word: {}: test passed\n dfa check: {}, nfa check: {}, afa check: {}, re check: {}, ext re check: {}",
                new_string,
                res_dfa_check,
                res_nfa_check,
                res_afa_check,
                res_re_match,
                res_ext_re_match
            );
        } else {
            error!(
                "word: {}: test not passed\n dfa check: {}, nfa check: {}, afa check: {}, re check: {}, ext re check: {}",
                new_string,
                res_dfa_check,
                res_nfa_check,
                res_afa_check,
                res_re_match,
                res_ext_re_match
            )
        }
    }

    if test_passed == new_fuzzer.tests_count {
        info!(
            "All tests passed: {}/{}",
            test_passed, new_fuzzer.tests_count
        ) //И да, я знаю что переменные равны и я мог использовать одну переменную для вывода или вообще не выводить, но хочу так
    } else {
        error!(
            "Not all tests passed: {}/{}",
            test_passed, new_fuzzer.tests_count
        )
    }
}

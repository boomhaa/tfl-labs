use log::{error, info};
use rand::Rng;

struct Rule {
    left_rule: String,
    right_rule: String,
}

struct MetaTest {
    tests_count: usize,
    min_str_len: usize,
    max_str_len: usize,
    max_rewrites: usize,
    alphabet: Vec<char>,
    base_rules: Vec<Rule>,
    new_rules: Vec<Rule>,
    rnd: rand::rngs::ThreadRng,
}

impl MetaTest {
    fn new() -> Self {
        Self {
            tests_count: 3333,
            min_str_len: 10,
            max_str_len: 100,
            max_rewrites: 50,
            alphabet: vec!['a', 'b', 'c'],
            rnd: rand::thread_rng(),
            base_rules: vec![
                Rule {
                    left_rule: "cb".to_string(),
                    right_rule: "ba".to_string(),
                },
                Rule {
                    left_rule: "aaa".to_string(),
                    right_rule: "aa".to_string(),
                },
                Rule {
                    left_rule: "aba".to_string(),
                    right_rule: "ba".to_string(),
                },
                Rule {
                    left_rule: "cc".to_string(),
                    right_rule: "ac".to_string(),
                },
                Rule {
                    left_rule: "baa".to_string(),
                    right_rule: "ba".to_string(),
                },
                Rule {
                    left_rule: "bba".to_string(),
                    right_rule: "ba".to_string(),
                },
                Rule {
                    left_rule: "bbb".to_string(),
                    right_rule: "b".to_string(),
                },
                Rule {
                    left_rule: "bbc".to_string(),
                    right_rule: "c".to_string(),
                },
                Rule {
                    left_rule: "bcc".to_string(),
                    right_rule: "cc".to_string(),
                },
                Rule {
                    left_rule: "cab".to_string(),
                    right_rule: "ba".to_string(),
                },
                Rule {
                    left_rule: "cac".to_string(),
                    right_rule: "cc".to_string(),
                },
                Rule {
                    left_rule: "cac".to_string(),
                    right_rule: "bab".to_string(),
                },
                Rule {
                    left_rule: "ccc".to_string(),
                    right_rule: "c".to_string(),
                },
                Rule {
                    left_rule: "babb".to_string(),
                    right_rule: "ba".to_string(),
                },
                Rule {
                    left_rule: "cabba".to_string(),
                    right_rule: "baca".to_string(),
                },
                Rule {
                    left_rule: "caab".to_string(),
                    right_rule: "bb".to_string(),
                },
                Rule {
                    left_rule: "caac".to_string(),
                    right_rule: "bc".to_string(),
                },
                Rule {
                    left_rule: "aabcaa".to_string(),
                    right_rule: "a".to_string(),
                },
                Rule {
                    left_rule: "babc".to_string(),
                    right_rule: "".to_string(),
                },
            ],
            new_rules: vec![
                Rule {
                    left_rule: "c".to_string(),
                    right_rule: "a".to_string(),
                },
                Rule {
                    left_rule: "b".to_string(),
                    right_rule: "a".to_string(),
                },
            ],
        }
    }

    fn gen_string(&mut self) -> String {
        let mut string = String::new();
        let length = self.rnd.gen_range(self.min_str_len..=self.max_str_len);

        for _ in 0..length {
            string.push(self.alphabet[self.rnd.gen_range(0..self.alphabet.len())])
        }

        string
    }
    fn random_rewrite(&mut self, string: &String, base: bool) -> Option<String> {
        let mut new_string = string.clone();
        let count_rewrites = self.rnd.gen_range(0..self.max_rewrites);

        let rules = if base {
            &self.base_rules
        } else {
            &self.new_rules
        };

        for _ in 0..count_rewrites {
            let mut entries = Vec::new();
            for (rule_id, rule) in rules.iter().enumerate() {
                if rule.left_rule.is_empty() {
                    continue;
                }
                let mut start = 0;
                while let Some(pos) = new_string[start..].find(&rule.left_rule) {
                    entries.push((rule_id, start + pos));
                    start = start + pos + 1;

                    if start >= new_string.len() {
                        break;
                    }
                }
            }
            if entries.is_empty() {
                break;
            }
            let rand_rewrite = entries[self.rnd.gen_range(0..entries.len())];
            new_string.replace_range(
                rand_rewrite.1..rand_rewrite.1 + rules[rand_rewrite.0].left_rule.len(),
                &rules[rand_rewrite.0].right_rule,
            );
        }
        Some(new_string)
    }

    fn capitalize(&self, s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    fn count_parikh_measure(&self, word: &str) -> usize {
        word.chars().filter(|&ch| ch == 'a').count()
            + word.chars().filter(|&ch| ch == 'b').count()
            + 2 * word.chars().filter(|&ch| ch == 'c').count()
    }

    fn start_weighted_parikh_measure_invariant_tests(&mut self, base: bool) {
        let system = if base {
            "base".to_string()
        } else {
            "new".to_string()
        };
        info!(
            "Testing {} system (strictly decreasing Parikh measure F(w) = #a + #b + 2*#c)...",
            system
        );

        for _ in 0..self.tests_count {
            let mut gen_string = self.gen_string();
            let mut p_measure = self.count_parikh_measure(&gen_string);
            for _ in 0..self.max_rewrites {
                if let Some(new) = self.random_rewrite(&gen_string, base) {
                    let new_p_measure = self.count_parikh_measure(&new);
                    if new_p_measure > p_measure {
                        error!(
                            "Invariant fail: {gen_string} → {new} (#c {p_measure} → {new_p_measure})"
                        );
                        break;
                    }
                    gen_string = new;
                    p_measure = new_p_measure;
                } else {
                    break;
                }
            }
        }
        info!(
            "{} system OK: Parikh measure decreases",
            self.capitalize(&system)
        )
    }

    fn start_m_invariant_tests(&mut self, base: bool) {
        let system = if base {
            "base".to_string()
        } else {
            "new".to_string()
        };
        info!(
            "Testing {} system (strictly decreasing det = α^#c)...",
            system
        );
        for _ in 0..self.tests_count {
            let mut gen_string = self.gen_string();
            let mut count_c = gen_string.chars().filter(|&ch| ch == 'c').count();
            for _ in 0..self.max_rewrites {
                if let Some(new) = self.random_rewrite(&gen_string, base) {
                    let new_count_c = new.chars().filter(|&ch| ch == 'c').count();
                    if new_count_c > count_c {
                        error!(
                            "Invariant fail: {gen_string} → {new} (#c {count_c} → {new_count_c})"
                        );
                        break;
                    }
                    gen_string = new;
                    count_c = new_count_c;
                } else {
                    break;
                }
            }
        }
        info!("{} system OK: det decreases", self.capitalize(&system));
    }
}

pub fn start_meta_tests() {
    let mut meta_tester = MetaTest::new();

    //start tests with M invariant
    meta_tester.start_m_invariant_tests(true);
    meta_tester.start_m_invariant_tests(false);
    
    //start tests with Parikh measure invariant
    meta_tester.start_weighted_parikh_measure_invariant_tests(true);
    meta_tester.start_weighted_parikh_measure_invariant_tests(false);
}

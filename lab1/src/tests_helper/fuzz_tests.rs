use csv::{WriterBuilder};
use log::{error, info};
use rand::Rng;

struct Rule {
    left_rule: String,
    right_rule: String,
}
struct Fuzzer {
    tests_count: usize,
    min_str_len: usize,
    max_str_len: usize,
    max_rewrites: usize,
    alphabet: Vec<char>,
    rules: Vec<Rule>,
    rnd: rand::rngs::ThreadRng,
}

impl Fuzzer {
    fn new() -> Self {
        Self {
            tests_count: 3333,
            min_str_len: 10,
            max_str_len: 100,
            max_rewrites: 50,
            alphabet: vec!['a', 'b', 'c'],
            rnd: rand::thread_rng(),
            rules: vec![
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

    fn random_rewrite(&mut self, string: &String) -> (String, usize) {
        let mut new_string = string.clone();
        let count_rewrites = self.rnd.gen_range(0..self.max_rewrites);

        for _ in 0..count_rewrites {
            let mut entries = Vec::new();
            for (rule_id, rule) in self.rules.iter().enumerate() {
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
                rand_rewrite.1..rand_rewrite.1 + self.rules[rand_rewrite.0].left_rule.len(),
                &self.rules[rand_rewrite.0].right_rule,
            );
        }
        (new_string, count_rewrites)
    }

    fn find_lcs(&self, string1: &String, string2: &String) -> usize {
        let chars1: Vec<char> = string1.chars().collect();
        let chars2: Vec<char> = string2.chars().collect();
        let m = chars1.len();
        let n = chars2.len();

        let mut dp = vec![vec![0usize; n + 1]; m + 1];
        for (i, char1) in chars1.iter().enumerate() {
            for (j, char2) in chars2.iter().enumerate() {
                if char1 == char2 {
                    dp[i + 1][j + 1] = dp[i][j] + 1;
                } else {
                    dp[i + 1][j + 1] = dp[i][j + 1].max(dp[i + 1][j]);
                }
            }
        }

        let length = dp[m][n];
        length
    }
}

pub fn start_fuzzer() {
    let mut fuzzer = Fuzzer::new();

    let mut file = match WriterBuilder::new()
        .delimiter(b';')
        .from_path("data/fuzzer_results.csv")
    {
        Ok(f) => {
            info!("Trying to open file data/fuzzer_results.csv");
            f
        }
        Err(e) => {
            error!("Error while open file {e}");
            return;
        }
    };

    match file.write_record(&[
        "original",
        "rewritten",
        "lcs",
        "steps_original",
        "steps_rewritten",
        "rewrites_count",
    ]) {
        Ok(_) => {
        }
        Err(e) => {
            error!("Error while writing to file {e}");
            return;
        }
    }

    for _ in 0..fuzzer.tests_count {
        let gen_string = fuzzer.gen_string();
        let (new_string, count_rewrites) = fuzzer.random_rewrite(&gen_string);
        let lcs = fuzzer.find_lcs(&gen_string, &new_string);
        match file.write_record(&[
            &gen_string,
            &new_string,
            &lcs.to_string(),
            &(gen_string.len() - lcs).to_string(),
            &(new_string.len() - lcs).to_string(),
            &count_rewrites.to_string(),
        ]) {
            Ok(_) => {
            }
            Err(e) => {
                error!("Error while writing to file {e}");
                return;
            }
        }
    }
    file.flush().expect("panic!");
    println!("Results saved to fuzzer_results.csv");
}

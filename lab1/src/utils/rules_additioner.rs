use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use log::{info, warn, error};

#[derive(Debug)]
struct RulesAddition {
    letters: HashMap<usize, String>,
    right_rules: Vec<String>,
    left_rules: Vec<String>,
    cur_len: usize,
    max_len: usize,
    alphabet_len: usize,
    error: bool,
    history: HashMap<String, Vec<String>>,
}

impl RulesAddition {
    fn new() -> Self {
        Self {
            letters: HashMap::new(),
            right_rules: vec![],
            left_rules: vec![],
            cur_len: 0,
            max_len: 0,
            alphabet_len: 0,
            error: false,
            history: HashMap::new(),
        }
    }

    fn read_letters(&mut self) {
        info!("Trying to open file data/alphabet.txt");
        let file = match File::open("data/alphabet.txt") {
            Ok(f) => {
                info!("File data/alphabet.txt opened successfully");
                f
            }
            Err(e) => {
                error!("Error while open file {e}");
                self.error = true;
                return;
            }
        };

        let reader = BufReader::new(file);
        let mut letters = vec![];
        for (i, line) in reader.lines().enumerate() {
            if i == 0 {
                match line.unwrap().parse::<usize>() {
                    Ok(n) => self.max_len = n,
                    Err(e) => {
                        self.error = true;
                        error!("Parse max len with error: {e}");
                        return;
                    }
                }
            } else {
                let letter = line.unwrap();
                letters.push(letter);
            }
        }

        letters.sort_by(|a, b| {
            if a.len() != b.len() {
                a.len().cmp(&b.len())
            } else {
                a.cmp(b)
            }
        });

        for (i, letter) in letters.into_iter().enumerate() {
            self.letters.insert(i, letter);
        }

        self.alphabet_len = self.letters.len();
    }

    fn read_rules(&mut self) {
        info!("Trying to open file data/rules.txt");
        let file = match File::open("data/rules.txt") {
            Ok(f) => {
                info!("File data/rules.txt opened successfully");
                f
            }
            Err(e) => {
                error!("Error while open file {e}");
                self.error = true;
                return;
            }
        };

        let reader = BufReader::new(file);
        self.left_rules.clear();
        self.right_rules.clear();
        info!("-------------SRS----------------");
        for line in reader.lines() {
            let rule = line.unwrap();
            if rule.is_empty() {
                continue;
            }
            if let Some(index) = rule.find(" -> ") {
                let left = &rule[..index];
                let mut right = &rule[index + 4..];
                if right == "." {
                    right = ""
                }
                info!("{} -> {}", left, right);
                self.add_rules(left, right)
            }
        }
        info!("--------------------------------");
    }

    fn add_rules(&mut self, left_rule: &str, right_rule: &str) {
        let size = self.left_rules.len();
        self.left_rules.push(left_rule.to_string());
        self.right_rules.push(right_rule.to_string());

        for i in 0..size {
            let left_clone = self.left_rules[i].clone();
            let normal_forms = self.get_normal_forms(&left_clone, vec![]);
            if normal_forms[0] != self.right_rules[i] {
                self.right_rules[i] = normal_forms[0].clone();
            }
        }
    }

    fn get_normal_forms(&mut self, start: &str, mut history: Vec<String>) -> Vec<String> {
        history.push(start.to_string());
        let mut normal_forms = vec![];
        let mut is_normal_form = true;

        let rules: Vec<(String, String)> = self
            .left_rules
            .iter()
            .cloned()
            .zip(self.right_rules.iter().cloned())
            .collect();

        for (left_rule, right_rule) in rules {
            let indexes = Self::find_terms(start, &left_rule);
            if !indexes.is_empty() {
                is_normal_form = false;
                for index in indexes {
                    let mut new_start = start.to_string();
                    new_start.replace_range(index..index + left_rule.len(), &right_rule);

                    let new_history = history.clone();
                    let more_normal_forms = self.get_normal_forms(&new_start, new_history);

                    for form in more_normal_forms {
                        if !normal_forms.contains(&form) {
                            normal_forms.push(form);
                        }
                    }
                }
            }
        }

        if is_normal_form {
            normal_forms.push(start.to_string());
            self.history
                .insert(history.last().unwrap().clone(), history);
        }

        normal_forms
    }

    fn find_terms(string: &str, term: &str) -> Vec<usize> {
        let mut result = vec![];
        let mut pos = string.find(term);
        while let Some(i) = pos {
            result.push(i);
            pos = string[i + 1..].find(term).map(|x| x + i + 1);
        }
        result
    }

    fn reduction_rules(&mut self) {
        let mut i = 0;
        while i < self.left_rules.len() {
            let start = self.left_rules[i].clone();
            let left = self.left_rules[i].clone();
            let right = self.right_rules[i].clone();

            self.left_rules.remove(i);
            self.right_rules.remove(i);

            let normal_forms = self.get_normal_forms(&start, vec![]);

            if !normal_forms.contains(&right) {
                self.left_rules.insert(i, left);
                self.right_rules.insert(i, right);
            }
            i += 1;
        }
    }

    fn gen_string(&self, number: usize, length: usize) -> String {
        if length == 0 {
            return String::new();
        }
        let prev = self.gen_string(number / self.alphabet_len, length - 1);
        let letter = self.letters.get(&(number % self.alphabet_len)).unwrap();
        format!("{}{}", prev, letter)
    }
}

pub fn start_rules_additioner() {
    env_logger::init();
    let mut rules_addition = RulesAddition::new();
    rules_addition.read_letters();
    if rules_addition.error {
        return;
    }
    info!("Letters read");

    rules_addition.read_rules();
    info!("Rules read");

    rules_addition.reduction_rules();

    let mut cnt = 0;
    let mut to_add: HashMap<String, String> = HashMap::new();

    while rules_addition.cur_len <= rules_addition.max_len {
        loop {
            for i in 0..rules_addition
                .alphabet_len
                .pow(rules_addition.cur_len as u32)
            {
                let gen_string = rules_addition.gen_string(i, rules_addition.cur_len);
                rules_addition.history.clear();
                let normal_forms = rules_addition.get_normal_forms(&gen_string, vec![]);
                if normal_forms.len() != 1 {
                    cnt += 1;
                    warn!("{gen_string} has more, than 1 normal form");
                    for (key, val) in &rules_addition.history {
                        info!("{}: {:?}", key, val)
                    }
                    let mut sorted_normal_forms = normal_forms.clone();
                    sorted_normal_forms.sort_by(|a, b| {
                        if a.len() != b.len() {
                            a.len().cmp(&b.len())
                        } else {
                            a.cmp(&b)
                        }
                    });
                    for pair in sorted_normal_forms.windows(2) {
                        to_add.insert(pair[1].clone(), pair[0].clone());
                    }
                } else {
                    info!("{} norm: {} -> {}", gen_string, gen_string, normal_forms[0]);
                }
            }

            if cnt == 0 {
                info!("My job finished! Goodbye!");
                break;
            }

            for (key, val) in &to_add {
                info!("Added rule {key} -> {val}")
            }

            if to_add.is_empty() {
                break;
            }

            let mut file = File::create("data/rules.txt").unwrap();
            for (left, right) in rules_addition
                .left_rules
                .iter()
                .zip(rules_addition.right_rules.iter())
            {
                if !right.is_empty() {
                    writeln!(file, "{} -> {}", left, right).unwrap();
                } else {
                    writeln!(file, "{} -> .", left).unwrap();
                }
            }

            if let Some((left, right)) = to_add.iter().next() {
                if !right.is_empty() {
                    writeln!(file, "{} -> {}", left, right).unwrap();
                } else {
                    writeln!(file, "{} -> .", left).unwrap();
                }
                info!("{} -> {} written", left, right);
            }

            rules_addition.read_rules();
            rules_addition.reduction_rules();
            to_add.clear();
            cnt = 0;
        }
        rules_addition.cur_len += 1;
    }
}

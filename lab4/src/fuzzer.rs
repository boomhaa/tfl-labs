use std::io::Write;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;
use rand::Rng;
use rand::rngs::ThreadRng;
use log::{error, info};

struct Fuzzer {
    words_count_in_lang: usize,
    words_count_not_in_lang: usize,
    min_str_len: usize,
    max_str_len: usize,
    rng: ThreadRng,
}

impl Fuzzer {
    fn new() -> Self {
        Self {
            words_count_in_lang: 1000,
            words_count_not_in_lang: 1000,
            min_str_len: 100,
            max_str_len: 5000,
            rng: rand::rng(),
        }
    }

    fn eq_lookahead(subword_x: &[u8], subword_y: &[u8]) -> bool {
        let total = subword_x.len() + 2 + subword_y.len();
        for i in 0..total {
            let left_part = if i < subword_x.len() {
                subword_x[i]
            } else if i == subword_x.len() {
                b'a'
            } else if i == subword_x.len() + 1 {
                b'b'
            } else {
                subword_y[i - (subword_x.len() + 2)]
            };

            let right_part = if i < subword_y.len() {
                subword_y[i]
            } else if i == subword_y.len() {
                b'b'
            } else if i == subword_y.len() + 1 {
                b'a'
            } else {
                subword_x[i - (subword_y.len() + 2)]
            };

            if left_part != right_part {
                return false;
            }
        }
        true
    }

    fn gen_word_in_lang(&mut self, target_len: usize) -> String {

        /*
        Структура слова: xcyybax, где x и y - это подслова из (a|b)^*
        Длина слова в этом семействе:
        |x| = 2k + 1
        |y| = k
        |w| = |x| + 1 + 2|y| + 2 + |x|
            = (2k+1) + 1 + 2k + 2 + (2k+1) = 6k + 5

        => k = (|w|-5)/6

        Чтобы попадать в нужный диапазон, подберем k по target_len,
        а если target_len не кратен, просто подберем ближайший k

        комментарий писал сам, чтобы было понятно, как генерится слово из языка
         */
        let mut k = if target_len >= 5 { (target_len.saturating_sub(5)) / 6 } else { 1 };
        if k == 0 { k = 1; }

        let use_a = self.rng.random_bool(0.5);

        if use_a {
            let a = "a".repeat(k);
            let x = format!("{a}b{a}");
            let y = a.clone();
            format!("{x}c{y}{y}ba{x}")
        } else {
            let b = "b".repeat(k);
            let x = format!("{b}a{b}");
            let y = "b".repeat(k-1);
            format!("{x}c{y}{y}ba{x}")
        }
    }

    fn gen_word_not_in_lang(&mut self, target_len: usize) -> String {
        let word = self.gen_word_in_lang(target_len-1);
        let random_letter = match self.rng.random_range(0..3) {
            0 => 'a',
            1 => 'b',
            _ => 'c',
        };
        format!("{random_letter}{word}")
    }

    fn naive_parser(&mut self, word: &str) -> bool {
        let bytes_word = word.as_bytes();

        fn match_vec(bs: &[u8], pos: usize, pat: &Vec<u8>, idx: usize) -> Option<usize> {
            if idx == pat.len() {
                return Some(pos);
            }
            if pos >= bs.len() {
                return None;
            }
            if bs[pos] != pat[idx] {
                return None;
            }
            match_vec(bs, pos + 1, pat, idx + 1)
        }

        fn match_ba(bs: &[u8], pos: usize, step: usize) -> Option<usize> {
            if step == 2 {
                return Some(pos);
            }
            if pos >= bs.len() {
                return None;
            }
            let expected = if step == 0 { b'b' } else { b'a' };
            if bs[pos] != expected {
                return None;
            }
            match_ba(bs, pos + 1, step + 1)
        }

        fn parse_x_subword(bs: &[u8], pos: usize, x_subword: &mut Vec<u8>) -> bool {
            if pos < bs.len() && bs[pos] == b'c' {
                let mut y_subword = Vec::<u8>::new();
                if parse_y_subword(bs, pos + 1, x_subword, &mut y_subword) {
                    return true;
                }
            }

            if pos < bs.len() && (bs[pos] == b'a' || bs[pos] == b'b') {
                x_subword.push(bs[pos]);
                if parse_x_subword(bs, pos + 1, x_subword) {
                    return true;
                }
                x_subword.pop();
            }

            false
        }

        fn parse_y_subword(
            bs: &[u8],
            pos: usize,
            x_subword: &Vec<u8>,
            y_subword: &mut Vec<u8>,
        ) -> bool {
            if let Some(pos_after_y) = match_vec(bs, pos, y_subword, 0) {
                if let Some(pos_after_ba) = match_ba(bs, pos_after_y, 0) {
                    if let Some(pos_after_x) = match_vec(bs, pos_after_ba, x_subword, 0) {
                        if pos_after_x == bs.len() && Fuzzer::eq_lookahead(x_subword, y_subword) {
                            return true;
                        }
                    }
                }
            }

            if pos < bs.len() && (bs[pos] == b'a' || bs[pos] == b'b') {
                y_subword.push(bs[pos]);
                if parse_y_subword(bs, pos + 1, x_subword, y_subword) {
                    return true;
                }
                y_subword.pop();
            }

            false
        }

        let mut x_subword = Vec::<u8>::new();
        parse_x_subword(bytes_word, 0, &mut x_subword)
    }


    fn optimize_parser(&mut self, word: &str) -> bool {
        let bytes_word = word.as_bytes();
        let len_word = bytes_word.len();

        let mut letter_c_pos: Option<usize> = None;
        for (i, &ch) in bytes_word.iter().enumerate() {
            match ch {
                b'a' | b'b' => {}
                b'c' => {
                    if letter_c_pos.is_some() {
                        return false;
                    }
                    letter_c_pos = Some(i);
                }
                _ => return false,
            }
        }
        let letter_c = match letter_c_pos {
            Some(p) => p,
            None => return false,
        };

        let subword_x = &bytes_word[..letter_c];
        let subword_x_len = subword_x.len();

        if len_word < 2 * subword_x_len + 3 {
            return false;
        }

        let suffix_start = len_word - subword_x_len;
        if &bytes_word[suffix_start..] != subword_x {
            return false;
        }

        if suffix_start < 2 {
            return false;
        }
        if &bytes_word[suffix_start - 2..suffix_start] != b"ba" {
            return false;
        }

        let mid_start = letter_c + 1;
        let mid_end = suffix_start - 2;
        if mid_end < mid_start {
            return false;
        }
        let mid = &bytes_word[mid_start..mid_end];

        if !mid.iter().all(|&ch| ch == b'a' || ch == b'b') {
            return false;
        }

        if mid.len() % 2 != 0 {
            return false;
        }
        let half = mid.len() / 2;
        let subword_y = &mid[..half];
        if &mid[half..] != subword_y {
            return false;
        }

        Fuzzer::eq_lookahead(subword_x, subword_y)
    }
}

pub fn start_fuzzer() {
    let mut fuzzer = Fuzzer::new();

    let buckets_count = 30;
    let min_len = fuzzer.min_str_len;
    let max_len = fuzzer.max_str_len;
    let lengths_span = max_len - min_len + 1;

    let mut in_words_count_per_bucket = vec![0usize; buckets_count];
    let mut out_words_count_per_bucket = vec![0usize; buckets_count];

    let mut in_naive_time_sum_per_bucket = vec![0f64; buckets_count];
    let mut in_opt_time_sum_per_bucket = vec![0f64; buckets_count];
    let mut out_naive_time_sum_per_bucket = vec![0f64; buckets_count];
    let mut out_opt_time_sum_per_bucket = vec![0f64; buckets_count];

    let log_each_n_tests: usize = 50;

    let mut total_tests: usize = 0;
    let mut successful_tests: usize = 0;
    let mut failed_tests: usize = 0;

    for iteration_index in 0..fuzzer.words_count_in_lang {
        total_tests += 1;

        let target_len = fuzzer.rng.random_range(min_len..=max_len);
        let generated_word = fuzzer.gen_word_in_lang(target_len);
        let generated_word_len = generated_word.len();

        if iteration_index % log_each_n_tests == 0 {
            let preview_len = 120usize.min(generated_word_len);
            let preview = &generated_word[..preview_len];
            info!(
                "IN test {}: len={}, word_preview='{}{}'",
                iteration_index,
                generated_word_len,
                preview,
                if preview_len < generated_word_len { "..." } else { "" }
            );
        }

        let bucket_index = ((generated_word_len.saturating_sub(min_len)) * buckets_count) / lengths_span;
        let bucket_index = bucket_index.min(buckets_count - 1);

        let naive_start_time = Instant::now();
        let naive_result = fuzzer.naive_parser(&generated_word);
        let naive_elapsed = naive_start_time.elapsed();

        let opt_start_time = Instant::now();
        let opt_result = fuzzer.optimize_parser(&generated_word);
        let opt_elapsed = opt_start_time.elapsed();

        let mut ok = true;

        if naive_result != opt_result {
            ok = false;
            error!(
                "Mismatch (IN) on iter {}: naive={}, opt={}, len={}, w='{}'",
                iteration_index, naive_result, opt_result, generated_word_len, generated_word
            );
        }
        if !opt_result {
            ok = false;
            error!(
                "Generator produced NOT-IN word in IN pool on iter {}: len={}, w='{}'",
                iteration_index, generated_word_len, generated_word
            );
        }

        if ok {
            successful_tests += 1;
        } else {
            failed_tests += 1;
        }

        in_words_count_per_bucket[bucket_index] += 1;
        in_naive_time_sum_per_bucket[bucket_index] += naive_elapsed.as_secs_f64();
        in_opt_time_sum_per_bucket[bucket_index] += opt_elapsed.as_secs_f64();
    }

    for iteration_index in 0..fuzzer.words_count_not_in_lang {
        total_tests += 1;

        let target_len = fuzzer.rng.random_range(min_len..=max_len);
        let generated_word = fuzzer.gen_word_not_in_lang(target_len);
        let generated_word_len = generated_word.len();

        if iteration_index % log_each_n_tests == 0 {
            let preview_len = 120usize.min(generated_word_len);
            let preview = &generated_word[..preview_len];
            info!(
                "OUT test {}: len={}, word_preview='{}{}'",
                iteration_index,
                generated_word_len,
                preview,
                if preview_len < generated_word_len { "..." } else { "" }
            );
        }

        let bucket_index = ((generated_word_len.saturating_sub(min_len)) * buckets_count) / lengths_span;
        let bucket_index = bucket_index.min(buckets_count - 1);

        let naive_start_time = Instant::now();
        let naive_result = fuzzer.naive_parser(&generated_word);
        let naive_elapsed = naive_start_time.elapsed();

        let opt_start_time = Instant::now();
        let opt_result = fuzzer.optimize_parser(&generated_word);
        let opt_elapsed = opt_start_time.elapsed();

        let mut ok = true;

        if naive_result != opt_result {
            ok = false;
            error!(
                "Mismatch (OUT) on iter {}: naive={}, opt={}, len={}, w='{}'",
                iteration_index, naive_result, opt_result, generated_word_len, generated_word
            );
        }
        if opt_result {
            ok = false;
            error!(
                "Generator produced IN word in OUT pool on iter {}: len={}, w='{}'",
                iteration_index, generated_word_len, generated_word
            );
        }

        if ok {
            successful_tests += 1;
        } else {
            failed_tests += 1;
        }

        out_words_count_per_bucket[bucket_index] += 1;
        out_naive_time_sum_per_bucket[bucket_index] += naive_elapsed.as_secs_f64();
        out_opt_time_sum_per_bucket[bucket_index] += opt_elapsed.as_secs_f64();
    }

    write_csv(
        "in_times.csv",
        min_len,
        max_len,
        &in_words_count_per_bucket,
        &in_naive_time_sum_per_bucket,
        &in_opt_time_sum_per_bucket,
    );
    write_csv(
        "out_times.csv",
        min_len,
        max_len,
        &out_words_count_per_bucket,
        &out_naive_time_sum_per_bucket,
        &out_opt_time_sum_per_bucket,
    );

    info!("Fuzz finished.");
    info!(
        "Fuzzer summary: total_tests={}, successful_tests={}, failed_tests={}",
        total_tests, successful_tests, failed_tests
    );

    info!("Wrote info: in_times.csv and out_times.csv");
}

fn write_csv(
    filename: &str,
    min_len: usize,
    max_len: usize,
    counts: &[usize],
    naive_sum: &[f64],
    opt_sum: &[f64],
) {
    let buckets = counts.len();
    let span = max_len - min_len + 1;

    let file = File::create(filename).expect("Cannot create CSV");
    let mut w = BufWriter::new(file);

    writeln!(w, "bucket,len_from,len_to,count,avg_naive_sec,avg_opt_sec").unwrap();

    for b in 0..buckets {
        let len_from = min_len + (b * span) / buckets;
        let len_to = min_len + ((b + 1) * span) / buckets - 1;

        let cnt = counts[b];
        let avg_naive = if cnt == 0 { 0.0 } else { naive_sum[b] / cnt as f64 };
        let avg_opt = if cnt == 0 { 0.0 } else { opt_sum[b] / cnt as f64 };

        writeln!(
            w,
            "{},{},{},{},{:.10},{:.10}",
            b, len_from, len_to, cnt, avg_naive, avg_opt
        )
            .unwrap();
    }
}

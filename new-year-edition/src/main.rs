use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration};

fn progress_line(prefix: &str, msg: &str, delay_ms: u64) {
    print!("{prefix} {msg}");
    io::stdout().flush().ok();
    sleep(Duration::from_millis(delay_ms));
}

fn main() {
    let teacher = "Антонина Николаевна";
    let year = 2026;

    println!("\n🎉 С Новым годом, {teacher}!\n");
    println!("Пусть 2026 будет годом ясных идей и красивых решений");
    let frames = [
        r#"
           ✨
          /__\
         /____\
        /______\
           ||
        "#,
        r#"
           ✨
          /__\
         /_🎁_\
        /______\
           ||
        "#,
        r#"
           ⭐️
          /__\
         /_❄️_\
        /______\
           ||
        "#,
    ];

    for _ in 0..2 {
        for f in &frames {
            print!("\x1B[2J\x1B[H");
            println!("{f}");
            io::stdout().flush().ok();
            sleep(Duration::from_millis(450));
        }
    }

    println!("\nRelease notes: v{year}.0\n");
    progress_line("Added:", "больше понимания сложных тем", 300);
    progress_line("Improved:", "умение задавать правильные вопросы", 300);
    progress_line("Fixed:", "страх перед контрольными (частично)", 300);
    progress_line("Planned:", "ещё больше практики и аккуратного кода", 300);

}

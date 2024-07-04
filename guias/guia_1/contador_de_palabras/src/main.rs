use std::io::{self, BufRead, BufReader};
use std::{collections::HashMap, env, fs::File};

fn main() -> io::Result<()> {
    let phrases_path = env::args()
        .nth(1)
        .expect("Use: cargo run -- <phrases_path>");

    let reader = BufReader::new(File::open(phrases_path)?);
    let mut words = HashMap::<_, usize>::new();

    for line in reader.lines().map_while(Result::ok) {
        for word in line
            .split_whitespace()
            .map(|line| line.trim().to_lowercase())
        {
            let freq = words.entry(word).or_default();
            *freq += 1;
        }
    }

    let mut words: Vec<_> = words.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));

    for (word, freq) in words {
        println!("{word} -> {freq}");
    }

    Ok(())
}

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

#[allow(dead_code)]
fn print_hash<K, V, S>(hash: &HashMap<K, V, S>)
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    for (k, v) in hash {
        println!("{k} -> {v}");
    }
}

fn print_vec(v: Vec<(&String, &i8)>) {
    v.iter().for_each(|(s, i)| {
        println!("{s} -> {i}");
    });
    print!("\n\n\n");
}

fn main() -> Result<()> {
    let file = File::open("src/frases.txt")?;
    let reader = BufReader::new(file);

    reader.lines().for_each(|line| match line {
        Ok(l) => {
            let mut hash = HashMap::<String, i8>::new();
            l.to_lowercase().split(" ").for_each(|word| {
                if let Some(count) = hash.get(word) {
                    hash.insert(word.to_owned(), count + 1_i8);
                } else {
                    hash.insert(word.to_owned(), 1);
                }
            });

            let mut words: Vec<(&String, &i8)> = hash.iter().collect();
            words.sort_by_key(|(_, v)| -(*v));
            print_vec(words);
        }

        Err(_) => panic!("No se que paso che"),
    });

    Ok(())
}

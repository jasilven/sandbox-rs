use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <path>", args[0]);
        return Ok(());
    }

    let mut words: HashMap<String, usize> = HashMap::new();
    let f = File::open(&args[1])?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = line?;
        let line = line.trim_end();
        for word in
            line.split(|ch: char| char::is_ascii_whitespace(&ch) || char::is_ascii_punctuation(&ch))
        {
            if word.parse::<isize>().is_ok() || (word.len() <= 1) {
                continue;
            }
            let entry = words.entry(word.to_lowercase()).or_insert(0);
            *entry += 1;
        }
    }

    let mut sorted_words: Vec<(&String, &usize)> = words.iter().collect();
    sorted_words.sort_by_key(|(_, &cnt)| cnt);

    for (word, cnt) in sorted_words.iter().rev() {
        println!("{:>4}: {}", cnt, word);
    }

    Ok(())
}

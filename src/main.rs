use std::{fs::File, io::{BufReader, BufRead}, collections::HashMap};
use regex::Regex;
use substring::Substring;

fn main() {
    // const COLLECTION_PATH: &str = "C:\\Users\\Doug\\Documents\\Magic\\Collection.coll2";
    const DECK_PATHS: [&str; 8] = [
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Arcades.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Dragons.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\GroupHug.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Narset.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Neheb.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Rebels.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Stax.dec",
        "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Tasha.dec",
    ];
    let mut deck_contents:HashMap<String, u32> = HashMap::new();
    // let collection_contents:HashMap<String, u32> = HashMap::new();

    for path in DECK_PATHS {
        load_from_file(path, &mut deck_contents);
    }
    
    // load_from_file(COLLECTION_PATH, collection_contents);

    for (key, value) in deck_contents {
        println!("{}: {}", key, value);
    }
}

fn load_from_file(file_path: &str, contents: &mut HashMap<String, u32>) {
    let file = File::open(file_path).expect("Could not read file {file_path}");
    let reader = BufReader::new(file);
    let line_reg = Regex::new(r"/").unwrap();
    let quantity_reg = Regex::new(r"\d+").unwrap();

    for line in reader.lines().map(|line| line.unwrap().to_string()) {
        if !line_reg.is_match(&line) {
            let quantity_match = quantity_reg.find(&line).unwrap();
            let quantity = line.substring(quantity_match.start(), quantity_match.end()).parse::<u32>().unwrap();
            let card_name = line.substring(quantity_match.end() + 1, line.len()).to_string();

            if contents.contains_key(&card_name) {
                *contents.get_mut(&card_name).unwrap() += quantity;
            } else {
                contents.insert(card_name, quantity);
            }
        }
    }
}
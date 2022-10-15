use std::{fs::File, io::{BufReader, BufRead}, collections::HashMap};
use regex::Regex;
use substring::Substring;

fn main() {
    // const COLLECTION_PATH: &str = "C:\\Users\\Doug\\Documents\\Magic\\Collection.coll2";
    const DECK_PATH: &str = "C:\\Users\\Doug\\Documents\\Magic\\Decks\\Commander\\Narset.dec";
    let mut deck_contents:HashMap<&str, u32> = HashMap::new();
    
    load_from_file(DECK_PATH);


    // println!("{COLLECTION_PATH}");
    // println!("{DECK_PATH}");
    // print!("{deck_contents}");
}

fn load_from_file(file_path: &str) {
    let file = File::open(file_path).expect("Could not read file {file_path}");
    let reader = BufReader::new(file);
    let line_reg = Regex::new(r"/").unwrap();
    let quantity_reg = Regex::new(r"\d+").unwrap();
    let mut count = 0;

    //let lines = reader.lines().for_each(line!(println!("{line}")));

    for line in reader.lines().map(|line| line.unwrap().to_string()) {
        if !line_reg.is_match(&line) {
            println!("line: {line}");
            let quantity_match = quantity_reg.find(&line).unwrap();
            let quantity = line.substring(quantity_match.start(), quantity_match.end());
            let card_name = line.substring(quantity_match.end() + 1, line.len());
            println!("quantity: {quantity}");
            println!("name: {card_name}");

            // count += 1
        }
    }
    println!("total count: {count}");

    // Ok(())
    //print!("{lines}")
}
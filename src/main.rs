use std::{fs::File, io::{BufReader, BufRead}, collections::HashMap, error::Error, process};
use regex::Regex;
use substring::Substring;
use walkdir::{DirEntry, WalkDir};

static FORMATS: [&str; 3] = ["Standard", "Legacy", "Frontier"];
static TRACKED_MODERN_DECKS: [&str; 5] = ["Affinity.dec", "DeathsShadow-Grixis.dec", "GiftsStorm.dec", "Spirits-UW.dec", "Tron-G.dec"];

fn main() -> Result<(), Box<dyn Error>> {
    const COLLECTION_PATH: &str = "C:\\Users\\Doug\\Documents\\Magic\\collection.csv";
   
    let mut deck_contents:HashMap<String, u32> = HashMap::new();
    let mut collection_contents:HashMap<String, u32> = HashMap::new();
    let mut format = String::from("");

    let walker = WalkDir::new("C:\\Users\\Doug\\Documents\\Magic\\Decks").into_iter();
    for entry in walker.filter_entry(|e| !is_untracked_format(e)) {
        let file_name = String::from(entry?.file_name().to_string_lossy());

        if file_name.contains(".dec") {
            if format == "Modern" && TRACKED_MODERN_DECKS.contains(&&file_name[..]) {
                let file_path = format!("C:\\Users\\Doug\\Documents\\Magic\\Decks\\{format}\\{file_name}");
                load_deck_file(file_path, &mut deck_contents);
            } else if format != "Modern" {
                let file_path = format!("C:\\Users\\Doug\\Documents\\Magic\\Decks\\{format}\\{file_name}");
                load_deck_file(file_path, &mut deck_contents);
            }
        } else if file_name != "Decks" {
            format = file_name.to_owned();
        }
    }
    
    if let Err(err) = load_collection_file(COLLECTION_PATH, &mut collection_contents) {
        println!("{}", err);
        process::exit(1);
    }

    for (card_name, quantity) in deck_contents {
        let needed_quantity;

        if collection_contents.contains_key(&card_name) {
            let owned_quantity = *collection_contents.get_mut(&card_name).unwrap();
            needed_quantity = owned_quantity as i32 - quantity as i32;
        } else {
            needed_quantity = quantity as i32;
           println!("Missing {needed_quantity} {card_name}")
        }

        if needed_quantity < 0 {
            println!("Missing {} {}", needed_quantity.abs(), card_name)
        }
    }

    Ok(())
}

fn load_deck_file(file_path: String, contents: &mut HashMap<String, u32>) {
    let file = File::open(file_path).expect("Could not read file {file_path}");
    let reader = BufReader::new(file);
    let line_reg = Regex::new(r"/").unwrap();
    let quantity_reg = Regex::new(r"\d+").unwrap();

    for line in reader.lines().map(|line| line.unwrap().to_string()) {
        if !line_reg.is_match(&line) {
            let quantity_match = quantity_reg.find(&line).unwrap();
            let quantity = line.substring(quantity_match.start(), quantity_match.end()).parse::<u32>().unwrap();
            let card_name = line.substring(quantity_match.end() + 1, line.len()).to_string();

            set_hash(card_name, quantity, contents);
        }
    }
}

fn load_collection_file(file_path: &str, contents: &mut HashMap<String, u32>) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    for result in rdr.records() {
        let record = result?;
        let quantity = record[0].parse::<u32>().unwrap();
        let card_name = record[3].to_string();
        
        set_hash(card_name, quantity, contents);
    }
    Ok(())
}

fn set_hash(card_name: String, quantity: u32, contents: &mut HashMap<String, u32> ) {
    if contents.contains_key(&card_name) {
        *contents.get_mut(&card_name).unwrap() += quantity;
    } else {
        contents.insert(card_name, quantity);
    }
}

fn is_untracked_format(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| FORMATS.contains(&s) || s.starts_with("."))
         .unwrap_or(false)
}

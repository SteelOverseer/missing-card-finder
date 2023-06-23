use std::{fs::{File, self}, io::{BufReader, BufRead, Write}, collections::HashMap, error::Error, process, path::Path};
use regex::Regex;
use substring::Substring;
mod configuration;

#[derive(Debug)]
struct CollectionCard {
    total_qty: u64,
    reg_qty: u64,
    foil_qty: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");   
    let mut collection_contents:HashMap<String, CollectionCard> = HashMap::new();

    if let Err(err) = load_collection_file(&configuration.collection_path, &mut collection_contents, &configuration.excluded_cards) {
        println!("{}", err);
        process::exit(1);
    }

    // Create output file
    if Path::new(&configuration.output_path).exists() {
        fs::remove_file(&configuration.output_path).unwrap();
    }
    let mut output_file = File::create(&configuration.output_path)?;

    for format in &configuration.tracked_formats {
        writeln!(output_file, "/////////////////////////////////// {format} ///////////////////////////////////")?;
        if format == "Commander" {
            for deck in &configuration.tracked_commander_decks {
                
                if configuration.foil_decks.contains(&deck) {
                    writeln!(output_file, "-------- {deck} ** FOIL ** --------")?;    
                } else {
                    writeln!(output_file, "-------- {deck} --------")?;
                }
                
                let deck_contents = load_deck_file(format, deck, &configuration.decks_path, &configuration.excluded_cards);
                for (card_name, quantity) in &deck_contents {
                    let (needed_quantity, name) = process_deck(card_name, quantity, deck, &mut collection_contents, &configuration.foil_decks);

                    if needed_quantity > 0 {
                        writeln!(output_file, "{needed_quantity} {name}")?;
                    }
                }
            }
        } else if format == "Modern" {
            for deck in &configuration.tracked_modern_decks {
                
                if configuration.foil_decks.contains(&deck) {
                    writeln!(output_file, "-------- {deck} ** FOIL ** --------")?;    
                } else {
                    writeln!(output_file, "-------- {deck} --------")?;
                }

                let deck_contents = load_deck_file(format, deck, &configuration.decks_path, &configuration.excluded_cards);
                for (card_name, quantity) in &deck_contents {
                    let (needed_quantity, name) = process_deck(card_name, quantity, deck, &mut collection_contents, &configuration.foil_decks);

                    if needed_quantity > 0 {
                        writeln!(output_file, "{needed_quantity} {name}")?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn load_deck_file<'a>(format: &'a str, deck: &String, deck_path: &String, excluded_cards: &Vec<String>) -> HashMap<String, u64> {
    let file_path = format!("{}\\{}\\{}.dec", deck_path, format, deck);
    let file = File::open(file_path).expect("Could not read file {file_path}");
    let reader = BufReader::new(file);
    let line_reg = Regex::new(r"/").unwrap(); // .dec files have lines that start with //, i dont need these lines
    let quantity_reg = Regex::new(r"\d+").unwrap();
    let mut deck_contents:HashMap<String, u64> = HashMap::new();

    for line in reader.lines().map(|line| line.unwrap().to_string()) {
        if !line_reg.is_match(&line) {
            let quantity_match = quantity_reg.find(&line).unwrap();
            let quantity = line.substring(quantity_match.start(), quantity_match.end()).parse::<u64>().unwrap();
            let card_name = line.substring(quantity_match.end() + 1, line.len()).to_string();

            set_hash(card_name, quantity, &mut deck_contents, &excluded_cards);
        }
    }

    return deck_contents;
}

fn process_deck(card_name: &String, quantity: &u64, deck: &String, collection_contents: &mut HashMap<String, CollectionCard>, foil_decks: &Vec<String>) -> (i32, String) {
    let needed_quantity;
    let owned_quantity;

    if collection_contents.contains_key(card_name.as_str()) {
        if foil_decks.contains(&deck) {
            owned_quantity = collection_contents.get_mut(card_name.as_str()).unwrap().foil_qty;

            if owned_quantity > 0 {
                if quantity > &collection_contents.get_mut(card_name.as_str()).unwrap().foil_qty {
                    collection_contents.get_mut(card_name.as_str()).unwrap().foil_qty = 0;
                } else {
                    collection_contents.get_mut(card_name.as_str()).unwrap().foil_qty -= quantity;
                }
            }
        } else {
            owned_quantity = collection_contents.get_mut(card_name.as_str()).unwrap().total_qty;
        }
        
        needed_quantity = owned_quantity as i32 - *quantity as i32;

        if owned_quantity > 0 {
            if quantity > &collection_contents.get_mut(card_name.as_str()).unwrap().total_qty {
                collection_contents.get_mut(card_name.as_str()).unwrap().total_qty = 0
            } else {
                collection_contents.get_mut(card_name.as_str()).unwrap().total_qty -= quantity;
            }
        }

        if needed_quantity < 0 {
            return (needed_quantity.abs(), card_name.to_string());
        } else {
            return (0, "There was an error with {card_name}".to_string())
        }

    } else {
        needed_quantity = *quantity as i32;
        return (needed_quantity, card_name.to_string());
    }
}

fn load_collection_file(file_path: &str, contents: &mut HashMap<String, CollectionCard>, excluded_cards: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    for result in rdr.records() {
        let record = result?;
        let total_quantity = record[0].parse::<u64>().unwrap();
        let regular_quantity = record[1].parse::<u64>().unwrap();
        let foil_quantity = record[2].parse::<u64>().unwrap();
        let card_name = record[3].split("//").next().unwrap().trim().to_string();
        
        if !excluded_cards.contains(&card_name) {
            if contents.contains_key(&card_name) {
                contents.get_mut(&card_name).unwrap().total_qty += total_quantity;
                contents.get_mut(&card_name).unwrap().reg_qty += regular_quantity;
                contents.get_mut(&card_name).unwrap().foil_qty += foil_quantity;
            } else {
                contents.insert(card_name, CollectionCard { total_qty: total_quantity, reg_qty: regular_quantity, foil_qty: foil_quantity });
            }
        }        
    }

    Ok(())
}

fn set_hash(card_name: String, quantity: u64, contents: &mut HashMap<String, u64>, excluded_cards: &Vec<String>) {
    if excluded_cards.contains(&card_name) {
        return;
    }

    if contents.contains_key(&card_name) {
        *contents.get_mut(&card_name).unwrap() += quantity;
    } else {
        contents.insert(card_name, quantity);
    }
}

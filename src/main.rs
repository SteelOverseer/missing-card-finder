use std::{fs::{File, self}, io::{BufReader, BufRead, Write}, collections::{HashMap, BTreeMap}, error::Error, process, path::Path};
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
    // Card names are compared in lowercase, so the exclusion list must be lowercase too
    let excluded_cards: Vec<String> = configuration.excluded_cards.iter().map(|card| card.to_ascii_lowercase()).collect();

    if let Err(err) = load_collection_file(&configuration.collection_path, &mut collection_contents, &excluded_cards) {
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

        let decks = match format.as_str() {
            "Commander" => &configuration.tracked_commander_decks,
            "Modern" => &configuration.tracked_modern_decks,
            _ => {
                eprintln!("Unknown format in tracked_formats: {format}");
                continue;
            }
        };

        for deck in decks {
            report_deck(&mut output_file, format, deck, &configuration, &excluded_cards, &mut collection_contents)?;
        }
    }

    if configuration.debug {
        writeln!(output_file, "DEBUG-- COLLECTION CONTENTS")?;
        for(cardname, card_info) in collection_contents {
            let total = card_info.total_qty;
            let foil = card_info.foil_qty;
            let reg = card_info.reg_qty;

            writeln!(output_file, "{cardname}: ALL {total}, FOIL {foil}, REG {reg}")?;
        }
    }

    Ok(())
}

fn report_deck(output_file: &mut File, format: &str, deck: &String, configuration: &configuration::Settings, excluded_cards: &Vec<String>, collection_contents: &mut HashMap<String, CollectionCard>) -> Result<(), Box<dyn Error>> {
    if configuration.foil_decks.contains(deck) {
        writeln!(output_file, "-------- {deck} ** FOIL ** --------")?;
    } else {
        writeln!(output_file, "-------- {deck} --------")?;
    }

    let deck_contents = load_deck_file(format, deck, &configuration.decks_path, excluded_cards);
    for (card_name, quantity) in &deck_contents {
        let needed_quantity = process_deck(card_name, *quantity, deck, collection_contents, &configuration.foil_decks);

        if needed_quantity > 0 {
            writeln!(output_file, "{needed_quantity} {card_name}")?;
        }

        if configuration.debug {
            writeln!(output_file, "DEBUG-- cardname: {card_name}, quantity: {quantity}")?;
        }
    }

    Ok(())
}

fn load_deck_file<'a>(format: &'a str, deck: &String, deck_path: &String, excluded_cards: &Vec<String>) -> BTreeMap<String, u64> {
    let file_path = format!("{}\\{}\\{}.dec", deck_path, format, deck);
    let file = File::open(file_path).expect("Could not read file {file_path}");
    let reader = BufReader::new(file);
    let line_reg = Regex::new(r"^/").unwrap(); // .dec files have lines that start with /, i dont need these lines
    let quantity_reg = Regex::new(r"\d+").unwrap();
    let split_reg = Regex::new(r"/").unwrap();
    let mut deck_contents:BTreeMap<String, u64> = BTreeMap::new();

    for line in reader.lines().map(|line| line.unwrap().to_string()) {
        if !line_reg.is_match(&line) {
            let quantity_match = quantity_reg.find(&line).unwrap();
            let quantity = line.substring(quantity_match.start(), quantity_match.end()).parse::<u64>().unwrap();
            let mut card_name = line.substring(quantity_match.end() + 1, line.len()).to_string();

            //Collection only has front name for cards split with "//"
            if card_name.contains("/") {
                let split_match = split_reg.find(&card_name).unwrap();
                card_name = card_name.substring(0, split_match.start() - 1).to_string();
            }

            set_hash(card_name, quantity, &mut deck_contents, &excluded_cards);
        }
    }

    return deck_contents;
}

/// Takes a deck's copies of a card out of the collection and returns how many are still missing.
fn process_deck(card_name: &str, quantity: u64, deck: &String, collection_contents: &mut HashMap<String, CollectionCard>, foil_decks: &Vec<String>) -> u64 {
    let Some(card) = collection_contents.get_mut(&card_name.to_ascii_lowercase()) else {
        return quantity;
    };

    let is_foil_deck = foil_decks.contains(deck);
    let owned_quantity = if is_foil_deck { card.foil_qty } else { card.total_qty };

    if is_foil_deck {
        card.foil_qty = card.foil_qty.saturating_sub(quantity);
    }

    // Intentional: a foil deck with no foil copies leaves the regular copies alone,
    // so they stay available for non-foil decks
    if owned_quantity > 0 {
        card.total_qty = card.total_qty.saturating_sub(quantity);
    }

    quantity.saturating_sub(owned_quantity)
}

fn load_collection_file(file_path: &str, contents: &mut HashMap<String, CollectionCard>, excluded_cards: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    for result in rdr.records() {
        let record = result?;
        let total_quantity = record[0].parse::<u64>().unwrap();
        let regular_quantity = record[1].parse::<u64>().unwrap();
        let foil_quantity = record[2].parse::<u64>().unwrap();
        let card_name = record[3].split("//").next().unwrap().trim().to_string().to_ascii_lowercase();
        
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

fn set_hash(card_name: String, quantity: u64, contents: &mut BTreeMap<String, u64>, excluded_cards: &Vec<String>) {
    if excluded_cards.contains(&card_name.to_ascii_lowercase()) {
        return;
    }

    if contents.contains_key(&card_name) {
        *contents.get_mut(&card_name).unwrap() += quantity;
    } else {
        contents.insert(card_name, quantity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collection_with(name: &str, total_qty: u64, reg_qty: u64, foil_qty: u64) -> HashMap<String, CollectionCard> {
        let mut collection = HashMap::new();
        collection.insert(name.to_string(), CollectionCard { total_qty, reg_qty, foil_qty });
        collection
    }

    fn foil_decks() -> Vec<String> {
        vec!["FoilDeck".to_string()]
    }

    #[test]
    fn card_not_in_collection_is_fully_missing() {
        let mut collection = collection_with("memnite", 4, 4, 0);
        let missing = process_deck("Ornithopter", 3, &"Affinity".to_string(), &mut collection, &foil_decks());
        assert_eq!(missing, 3);
    }

    #[test]
    fn owning_enough_copies_is_not_missing() {
        let mut collection = collection_with("memnite", 4, 4, 0);
        let missing = process_deck("Memnite", 4, &"Affinity".to_string(), &mut collection, &foil_decks());
        assert_eq!(missing, 0);
        assert_eq!(collection["memnite"].total_qty, 0);
    }

    #[test]
    fn owning_some_copies_reports_the_shortage() {
        let mut collection = collection_with("memnite", 1, 1, 0);
        let missing = process_deck("Memnite", 4, &"Affinity".to_string(), &mut collection, &foil_decks());
        assert_eq!(missing, 3);
        assert_eq!(collection["memnite"].total_qty, 0);
    }

    #[test]
    fn card_name_lookup_ignores_case() {
        let mut collection = collection_with("memnite", 4, 4, 0);
        let missing = process_deck("MEMNITE", 2, &"Affinity".to_string(), &mut collection, &foil_decks());
        assert_eq!(missing, 0);
    }

    #[test]
    fn decks_share_the_collection_in_order() {
        let mut collection = collection_with("memnite", 4, 4, 0);
        let first = process_deck("Memnite", 3, &"DeckA".to_string(), &mut collection, &foil_decks());
        let second = process_deck("Memnite", 3, &"DeckB".to_string(), &mut collection, &foil_decks());
        assert_eq!(first, 0);
        assert_eq!(second, 2);
    }

    #[test]
    fn foil_deck_counts_only_foil_copies() {
        let mut collection = collection_with("memnite", 4, 3, 1);
        let missing = process_deck("Memnite", 4, &"FoilDeck".to_string(), &mut collection, &foil_decks());
        assert_eq!(missing, 3);
    }

    #[test]
    fn foil_deck_uses_up_foil_and_total_copies() {
        let mut collection = collection_with("memnite", 4, 2, 2);
        let missing = process_deck("Memnite", 2, &"FoilDeck".to_string(), &mut collection, &foil_decks());
        assert_eq!(missing, 0);
        assert_eq!(collection["memnite"].foil_qty, 0);
        assert_eq!(collection["memnite"].total_qty, 2);
    }

    #[test]
    fn foil_deck_without_foils_leaves_regular_copies_available() {
        let mut collection = collection_with("memnite", 4, 4, 0);
        let foil_missing = process_deck("Memnite", 2, &"FoilDeck".to_string(), &mut collection, &foil_decks());
        assert_eq!(foil_missing, 2);
        assert_eq!(collection["memnite"].total_qty, 4);

        let regular_missing = process_deck("Memnite", 4, &"RegularDeck".to_string(), &mut collection, &foil_decks());
        assert_eq!(regular_missing, 0);
    }
}

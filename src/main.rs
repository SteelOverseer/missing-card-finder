use std::{fs::File, io::{BufReader, BufRead, Write}, collections::{HashMap, BTreeMap}, error::Error, process, path::Path};
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

    // File::create replaces any existing report
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
        eprintln!("DEBUG-- COLLECTION CONTENTS");
        for(cardname, card_info) in collection_contents {
            let total = card_info.total_qty;
            let foil = card_info.foil_qty;
            let reg = card_info.reg_qty;

            eprintln!("{cardname}: ALL {total}, FOIL {foil}, REG {reg}");
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

    let deck_contents = match load_deck_file(format, deck, &configuration.decks_path, excluded_cards) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Warning: skipping deck {deck}: {err}");
            writeln!(output_file, "-- SKIPPED: {err} --")?;
            return Ok(());
        }
    };
    for (card_name, quantity) in &deck_contents {
        let needed_quantity = process_deck(card_name, *quantity, deck, collection_contents, &configuration.foil_decks);

        if needed_quantity > 0 {
            writeln!(output_file, "{needed_quantity} {card_name}")?;
        }

        if configuration.debug {
            eprintln!("DEBUG-- cardname: {card_name}, quantity: {quantity}");
        }
    }

    Ok(())
}

fn load_deck_file<'a>(format: &'a str, deck: &String, deck_path: &String, excluded_cards: &Vec<String>) -> Result<BTreeMap<String, u64>, String> {
    let file_path = Path::new(deck_path).join(format).join(format!("{deck}.dec"));
    let file = File::open(&file_path).map_err(|err| format!("could not read {}: {err}", file_path.display()))?;
    let reader = BufReader::new(file);
    let mut deck_contents:BTreeMap<String, u64> = BTreeMap::new();

    for (index, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                eprintln!("Warning: stopped reading {} at line {}: {err}", file_path.display(), index + 1);
                break;
            }
        };
        let line = line.trim();

        // .dec files have lines that start with /, those are metadata and not cards
        if line.is_empty() || line.starts_with('/') {
            continue;
        }

        match parse_deck_line(line) {
            Some((quantity, card_name)) => set_hash(card_name, quantity, &mut deck_contents, excluded_cards),
            None => eprintln!("Warning: skipping line {} of {}: {line}", index + 1, file_path.display()),
        }
    }

    Ok(deck_contents)
}

/// Parses a "<quantity> <card name>" line. Collection only has the front name for split cards ("A // B")
fn parse_deck_line(line: &str) -> Option<(u64, String)> {
    let (quantity, card_name) = line.trim().split_once(char::is_whitespace)?;
    let quantity = quantity.parse::<u64>().ok()?;
    let card_name = card_name.split("//").next()?.trim();

    if card_name.is_empty() {
        return None;
    }

    Some((quantity, card_name.to_string()))
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

    for (index, result) in rdr.records().enumerate() {
        // +2: the header is row 1 and humans count from 1
        let row = index + 2;
        let record = match result {
            Ok(record) => record,
            Err(err) if err.is_io_error() => return Err(err.into()),
            Err(err) => {
                eprintln!("Warning: skipping collection row {row}: {err}");
                continue;
            }
        };

        let Some((card_name, total_quantity, regular_quantity, foil_quantity)) = parse_collection_record(&record) else {
            eprintln!("Warning: skipping collection row {row}: could not read quantities and card name");
            continue;
        };

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

/// Returns (lowercase card name, total qty, regular qty, foil qty). Collection only has the front name for split cards
fn parse_collection_record(record: &csv::StringRecord) -> Option<(String, u64, u64, u64)> {
    let total_quantity = record.get(0)?.trim().parse::<u64>().ok()?;
    let regular_quantity = record.get(1)?.trim().parse::<u64>().ok()?;
    let foil_quantity = record.get(2)?.trim().parse::<u64>().ok()?;
    let card_name = record.get(3)?.split("//").next()?.trim().to_ascii_lowercase();

    if card_name.is_empty() {
        return None;
    }

    Some((card_name, total_quantity, regular_quantity, foil_quantity))
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

    #[test]
    fn deck_line_with_quantity_and_name() {
        assert_eq!(parse_deck_line("4 Cranial Plating"), Some((4, "Cranial Plating".to_string())));
    }

    #[test]
    fn deck_line_split_card_keeps_front_name() {
        assert_eq!(parse_deck_line("1 Fire // Ice"), Some((1, "Fire".to_string())));
    }

    #[test]
    fn deck_line_with_extra_whitespace() {
        assert_eq!(parse_deck_line("  2   Island  "), Some((2, "Island".to_string())));
    }

    #[test]
    fn deck_line_that_is_not_a_card_is_rejected() {
        assert_eq!(parse_deck_line(""), None);
        assert_eq!(parse_deck_line("Island"), None);
        assert_eq!(parse_deck_line("x Island"), None);
        assert_eq!(parse_deck_line("4"), None);
        assert_eq!(parse_deck_line("4 // Ice"), None);
    }

    #[test]
    fn collection_record_is_parsed_and_lowercased() {
        let record = csv::StringRecord::from(vec!["2", "1", "1", "Fire // Ice", "Set"]);
        assert_eq!(parse_collection_record(&record), Some(("fire".to_string(), 2, 1, 1)));
    }

    #[test]
    fn collection_record_with_bad_quantity_is_rejected() {
        let record = csv::StringRecord::from(vec!["two", "1", "1", "Memnite"]);
        assert_eq!(parse_collection_record(&record), None);
    }

    #[test]
    fn collection_record_that_is_too_short_is_rejected() {
        let record = csv::StringRecord::from(vec!["2", "1"]);
        assert_eq!(parse_collection_record(&record), None);
    }
}

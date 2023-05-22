use std::{fs::{File, self}, io::{BufReader, BufRead, Write}, collections::HashMap, error::Error, process, path::Path};
use regex::Regex;
use substring::Substring;

static TRACKED_FORMATS: [&str; 2] = ["Modern", "Commander"];
static TRACKED_MODERN_DECKS: [&str; 5] = ["Affinity", "DeathsShadow-Grixis", "GiftsStorm", "Spirits-UW", "Tron-G"];
static TRACKED_COMMANDER_DECKS: [&str; 9] = ["Narset", "Neheb", "Rebels", "Dragons", "Arcades", "Stax", "GroupHug", "Squirrels", "Jhoira"]; 
static FOIL_DECKS: [&str; 2] = ["Narset", "Affinity"];
static EXCLUDED_CARDS: [&str; 20] = [
    "Plains", 
    "Island",
    "Swamp", 
    "Mountain", 
    "Forest", 
    "Snow-Covered Plains", 
    "Snow-Covered Island", 
    "Snow-Covered Swamp", 
    "Snow-Covered Mountain", 
    "Snow-Covered Forest",
    "Tundra",
    "Taiga",
    "Grim Monolith",
    "Plateau",
    "Scrubland",
    "Bayou",
    "Underground Sea",
    "Savannah",
    "Tropical Island",
    "Volcanic Island"
];

#[derive(Debug)]
struct CollectionCard {
    total_qty: u64,
    reg_qty: u64,
    foil_qty: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    const COLLECTION_PATH: &str = "D:\\Magic\\collection.csv";
    const OUTPUT_PATH: &str = "D:\\Magic\\MissingCards.txt";
   
    let mut collection_contents:HashMap<String, CollectionCard> = HashMap::new();

    if let Err(err) = load_collection_file(COLLECTION_PATH, &mut collection_contents) {
        println!("{}", err);
        process::exit(1);
    }

    // Create output file
    if Path::new(OUTPUT_PATH).exists() {
        fs::remove_file(OUTPUT_PATH).unwrap();
    }
    let mut output_file = File::create(OUTPUT_PATH)?;

    for format in TRACKED_FORMATS {
        writeln!(output_file, "/////////////////////////////////// {format} ///////////////////////////////////")?;
        if format == "Commander" {
            for deck in TRACKED_COMMANDER_DECKS {
                
                if FOIL_DECKS.contains(&deck) {
                    writeln!(output_file, "-------- {deck} ** FOIL ** --------")?;    
                } else {
                    writeln!(output_file, "-------- {deck} --------")?;
                }
                
                let deck_contents = load_deck_file(format, deck);
                for (card_name, quantity) in &deck_contents {
                    let (needed_quantity, name) = process_deck(card_name, quantity, deck, &mut collection_contents);

                    if needed_quantity > 0 {
                        writeln!(output_file, "{needed_quantity} {name}")?;
                    }
                }
            }
        } else if format == "Modern" {
            for deck in TRACKED_MODERN_DECKS {
                
                if FOIL_DECKS.contains(&deck) {
                    writeln!(output_file, "-------- {deck} ** FOIL ** --------")?;    
                } else {
                    writeln!(output_file, "-------- {deck} --------")?;
                }

                let deck_contents = load_deck_file(format, deck);
                for (card_name, quantity) in &deck_contents {
                    let (needed_quantity, name) = process_deck(card_name, quantity, deck, &mut collection_contents);

                    if needed_quantity > 0 {
                        writeln!(output_file, "{needed_quantity} {name}")?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn load_deck_file<'a>(format: &'a str, deck: &'a str) -> HashMap<String, u64> {
    let file_path = format!("D:\\Magic\\Decks\\{format}\\{deck}.dec");
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

            set_hash(card_name, quantity, &mut deck_contents);
        }
    }

    return deck_contents;
}

fn process_deck(card_name: &String, quantity: &u64, deck: &str, collection_contents: &mut HashMap<String, CollectionCard>) -> (i32, String) {
    let needed_quantity;
    let owned_quantity;

    if collection_contents.contains_key(card_name.as_str()) {
        if FOIL_DECKS.contains(&deck) {
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

fn load_collection_file(file_path: &str, contents: &mut HashMap<String, CollectionCard>) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    for result in rdr.records() {
        let record = result?;
        let total_quantity = record[0].parse::<u64>().unwrap();
        let regular_quantity = record[1].parse::<u64>().unwrap();
        let foil_quantity = record[2].parse::<u64>().unwrap();
        let card_name = record[3].split("//").next().unwrap().trim().to_string();
        
        if !EXCLUDED_CARDS.contains(&&card_name[..]) {
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

fn set_hash(card_name: String, quantity: u64, contents: &mut HashMap<String, u64> ) {
    if EXCLUDED_CARDS.contains(&&card_name[..]) {
        return;
    }

    if contents.contains_key(&card_name) {
        *contents.get_mut(&card_name).unwrap() += quantity;
    } else {
        contents.insert(card_name, quantity);
    }
}

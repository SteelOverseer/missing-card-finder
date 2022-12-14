use std::{fs::{File, self}, io::{BufReader, BufRead, Write}, collections::HashMap, error::Error, process, path::Path};
use regex::Regex;
use substring::Substring;
use walkdir::{DirEntry, WalkDir};

static EXCLUDE_FORMATS: [&str; 3] = ["Standard", "Legacy", "Frontier"];
static TRACKED_MODERN_DECKS: [&str; 5] = ["Affinity.dec", "DeathsShadow-Grixis.dec", "GiftsStorm.dec", "Spirits-UW.dec", "Tron-G.dec"];
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

fn main() -> Result<(), Box<dyn Error>> {
    const COLLECTION_PATH: &str = "C:\\Users\\Doug\\Documents\\Magic\\collection.csv";
    const OUTPUT_PATH: &str = "C:\\Users\\Doug\\Documents\\Magic\\MissingCards.txt";
   
    let mut deck_contents:HashMap<String, u32> = HashMap::new();
    let mut collection_contents:HashMap<String, u32> = HashMap::new();
    let mut format = String::from("");

    // walk through the Decks directory to find all .dec files
    let walker = WalkDir::new("C:\\Users\\Doug\\Documents\\Magic\\Decks").into_iter();
    for entry in walker.filter_entry(|e| !is_untracked_format(e)) {
        let file_name = String::from(entry?.file_name().to_string_lossy());

        // filename can be a file or a directory
        if file_name.contains(".dec") {
            // Filtering out decks that I do not want to track
            if format == "Modern" && TRACKED_MODERN_DECKS.contains(&&file_name[..]) {
                let file_path = format!("C:\\Users\\Doug\\Documents\\Magic\\Decks\\{format}\\{file_name}");
                load_deck_file(file_path, &mut deck_contents);
            } else if format != "Modern" {
                let file_path = format!("C:\\Users\\Doug\\Documents\\Magic\\Decks\\{format}\\{file_name}");
                load_deck_file(file_path, &mut deck_contents);
            }
        } else if file_name != "Decks" {
            // Set the format to this directory, I don't want to track decks for all formats
            format = file_name.to_owned();
        }
    }
    
    if let Err(err) = load_collection_file(COLLECTION_PATH, &mut collection_contents) {
        println!("{}", err);
        process::exit(1);
    }

    // Create output file
    if Path::new(OUTPUT_PATH).exists() {
        fs::remove_file(OUTPUT_PATH).unwrap();
    }
    let mut output_file = File::create(OUTPUT_PATH)?;

    for (card_name, quantity) in deck_contents {
        let needed_quantity;

        if collection_contents.contains_key(&card_name) {
            let owned_quantity = *collection_contents.get_mut(&card_name).unwrap();
            needed_quantity = owned_quantity as i32 - quantity as i32;
        } else {
            needed_quantity = quantity as i32;
            writeln!(output_file,"{needed_quantity} {card_name}")?;
        }

        if needed_quantity < 0 {
            writeln!(output_file,"{} {}", needed_quantity.abs(), card_name)?;
        }
    }

    Ok(())
}

fn load_deck_file(file_path: String, contents: &mut HashMap<String, u32>) {
    let file = File::open(file_path).expect("Could not read file {file_path}");
    let reader = BufReader::new(file);
    let line_reg = Regex::new(r"/").unwrap(); // .dec files have lines that start with //, i dont need these lines
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
    if EXCLUDED_CARDS.contains(&&card_name[..]) {
        return;
    }

    if contents.contains_key(&card_name) {
        *contents.get_mut(&card_name).unwrap() += quantity;
    } else {
        contents.insert(card_name, quantity);
    }
}

fn is_untracked_format(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| EXCLUDE_FORMATS.contains(&s) || s.starts_with("."))
         .unwrap_or(false)
}

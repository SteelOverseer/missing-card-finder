use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const COLLECTION_HEADER: &str = "Total Qty,Reg Qty,Foil Qty,Card,Set,Mana Cost,Card Type,Color,Rarity,Mvid,Single Price,Single Foil Price,Total Price,Price Source,Notes";

/// A throwaway working directory holding a config, a collection and decks.
/// The program reads configuration.yaml from its working directory, so each test gets its own.
struct Scenario {
    dir: TempDir,
}

impl Scenario {
    fn new(collection_rows: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let mut collection = vec![COLLECTION_HEADER];
        collection.extend_from_slice(collection_rows);
        fs::write(dir.path().join("collection.csv"), collection.join("\n")).unwrap();
        fs::create_dir_all(dir.path().join("decks/Modern")).unwrap();
        fs::create_dir_all(dir.path().join("decks/Commander")).unwrap();
        Scenario { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn write_deck(&self, format: &str, deck: &str, contents: &str) {
        fs::write(self.path().join("decks").join(format).join(format!("{deck}.dec")), contents).unwrap();
    }

    fn write_config(&self, modern_decks: &[&str], commander_decks: &[&str], foil_decks: &[&str], excluded_cards: &[&str]) {
        let list = |items: &[&str]| format!("[{}]", items.iter().map(|item| format!("\"{item}\"")).collect::<Vec<_>>().join(", "));
        let config = format!(
            "debug: false\n\
             collection_path: \"{root}/collection.csv\"\n\
             output_path: \"{root}/out.txt\"\n\
             decks_path: \"{root}/decks\"\n\
             tracked_formats: [Modern, Commander]\n\
             tracked_modern_decks: {modern}\n\
             tracked_commander_decks: {commander}\n\
             foil_decks: {foil}\n\
             excluded_cards: {excluded}\n",
            root = self.path().display(),
            modern = list(modern_decks),
            commander = list(commander_decks),
            foil = list(foil_decks),
            excluded = list(excluded_cards),
        );
        fs::write(self.path().join("configuration.yaml"), config).unwrap();
    }

    fn command(&self) -> Command {
        let mut command = Command::cargo_bin("missingcardfinder").unwrap();
        command.current_dir(self.path());
        command
    }

    fn report(&self) -> String {
        fs::read_to_string(self.path().join("out.txt")).unwrap()
    }
}

#[test]
fn reports_missing_cards_per_deck_in_config_order() {
    let scenario = Scenario::new(&[
        "4,4,0,Memnite,Set,0,Artifact Creature,Colorless,Common,1,,,,x,",
        "1,0,1,Ornithopter,Set,0,Artifact Creature,Colorless,Common,2,,,,x,",
        "1,1,0,Fire // Ice,Set,1R,Instant,Gold,Uncommon,3,,,,x,",
    ]);
    scenario.write_deck("Modern", "Affinity", "///mvid:1 qty:4 name:Memnite loc:Deck\n4 Memnite\n4 Ornithopter\n1 Gingerbrute\n2 Island\n");
    scenario.write_deck("Commander", "Narset", "1 Memnite\n1 Fire // Ice\n2 Fire // Ice\n");
    scenario.write_config(&["Affinity"], &["Narset"], &["Affinity"], &["Island"]);

    scenario.command().assert().success();

    // Affinity is a foil deck: only 1 foil Ornithopter is owned and no Memnite is foil.
    // Island is excluded. Narset then finds all 4 regular Memnite still available,
    // because the foil deck found no foils and left the regular copies alone.
    let expected = "\
/////////////////////////////////// Modern ///////////////////////////////////
-------- Affinity ** FOIL ** --------
1 Gingerbrute
4 Memnite
3 Ornithopter
/////////////////////////////////// Commander ///////////////////////////////////
-------- Narset --------
2 Fire
";
    assert_eq!(scenario.report(), expected);
}

#[test]
fn decks_share_the_collection_so_order_matters() {
    let scenario = Scenario::new(&["3,3,0,Memnite,Set,0,Artifact Creature,Colorless,Common,1,,,,x,"]);
    scenario.write_deck("Modern", "First", "2 Memnite\n");
    scenario.write_deck("Modern", "Second", "2 Memnite\n");
    scenario.write_config(&["First", "Second"], &[], &[], &[]);

    scenario.command().assert().success();

    assert!(scenario.report().contains("-------- First --------\n-------- Second --------\n1 Memnite\n"));
}

#[test]
fn bad_input_is_skipped_with_warnings() {
    let scenario = Scenario::new(&[
        "4,4,0,Memnite,Set,0,Artifact Creature,Colorless,Common,1,,,,x,",
        "abc,1,1,Broken Row,Set,0,Artifact,Colorless,Common,2,,,,x,",
    ]);
    scenario.write_deck("Modern", "Affinity", "///comment\n\n4 Memnite\nnonsense\n1 Gingerbrute\n");
    scenario.write_config(&["Affinity", "Typo"], &[], &[], &[]);

    scenario
        .command()
        .assert()
        .success()
        .stderr(predicate::str::contains("skipping collection row 3"))
        .stderr(predicate::str::contains("skipping line 4"))
        .stderr(predicate::str::contains("skipping deck Typo"));

    let report = scenario.report();
    assert!(report.contains("-------- Affinity --------\n1 Gingerbrute\n"));
    assert!(report.contains("-------- Typo --------\n-- SKIPPED:"));
}

#[test]
fn debug_output_goes_to_stderr_not_the_report() {
    let scenario = Scenario::new(&["1,1,0,Memnite,Set,0,Artifact Creature,Colorless,Common,1,,,,x,"]);
    scenario.write_deck("Modern", "Affinity", "1 Memnite\n");
    scenario.write_config(&["Affinity"], &[], &[], &[]);
    let config_path = scenario.path().join("configuration.yaml");
    let config = fs::read_to_string(&config_path).unwrap().replace("debug: false", "debug: true");
    fs::write(&config_path, config).unwrap();

    scenario.command().assert().success().stderr(predicate::str::contains("DEBUG--"));

    assert!(!scenario.report().contains("DEBUG--"));
}

#[test]
fn missing_collection_file_exits_with_an_error() {
    let scenario = Scenario::new(&[]);
    scenario.write_config(&[], &[], &[], &[]);
    fs::remove_file(scenario.path().join("collection.csv")).unwrap();

    scenario.command().assert().failure().code(1);
}

#[test]
fn missing_configuration_file_fails() {
    let scenario = Scenario::new(&[]);

    scenario.command().assert().failure();
}

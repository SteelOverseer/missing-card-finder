#[derive(serde::Deserialize)]
pub struct Settings {
    pub collection_path: String,
    pub output_path: String,
    pub decks_path: String,
    pub tracked_formats: Vec<String>,
    pub tracked_modern_decks: Vec<String>,
    pub tracked_commander_decks: Vec<String>,
    pub foil_decks: Vec<String>,
    pub excluded_cards: Vec<String>,
    pub debug: bool
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;

    settings.try_deserialize::<Settings>()
}
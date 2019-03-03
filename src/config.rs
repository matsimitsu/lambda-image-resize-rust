use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub sizes: Vec<f32>,
    pub replacements: Vec<(String, String)>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            sizes: Config::parse_sizes(),
            replacements: Config::parse_replacements(),
        }
    }

    pub fn parse_sizes() -> Vec<f32> {
        let mut sizes = Vec::new();
        if let Ok(sizes_string) = env::var("SIZES") {
            for size_string in sizes_string.split(',').into_iter() {
                sizes.push(size_string.parse().unwrap());
            }
        };

        sizes
    }

    pub fn parse_replacements() -> Vec<(String, String)> {
        let mut replacements = Vec::new();
        if let Ok(replacements_string) = env::var("REPLACEMENTS") {
            for replacement in replacements_string.split(',').into_iter() {
                let key_val: Vec<&str> = replacement.splitn(2, ":").collect();
                replacements.push((key_val[0].to_string(), key_val[1].to_string()));
            }
        };

        replacements
    }
}

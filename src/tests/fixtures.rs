use yaml_rust::YamlLoader;
use std::fs;

pub fn load_fixtures_from_yaml<T>(file_path: &str) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        T: std::convert::From<yaml_rust::Yaml>,
{
    let file_content = fs::read_to_string(file_path)?;

    let yaml_docs = YamlLoader::load_from_str(&file_content)?;
    let yaml_doc = &yaml_docs[0];

    let mut fixtures = Vec::new();
    if let yaml_rust::Yaml::Hash(hash_map) = yaml_doc {
        for (_, value) in hash_map {
            let fixture: T = T::from(value.clone());
            fixtures.push(fixture);
        }
    }

    Ok(fixtures)
}
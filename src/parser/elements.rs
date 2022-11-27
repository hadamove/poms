use std::collections::HashMap;

use lazy_static::lazy_static;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ElementData {
    pub symbol: String,
    pub vdw_radius: f32,
    pub jmol_color: [f32; 3],
}

impl Default for ElementData {
    fn default() -> Self {
        Self {
            symbol: "Unknown".to_string(),
            vdw_radius: 1.5,
            jmol_color: [1.0, 0.0, 1.0],
        }
    }
}

lazy_static! {
    static ref ELEMENTS: HashMap<String, ElementData> = {
        // Load element data into program memory during compile time.
        let data_json: &str = include_str!("./data/elements.json");
        let data: Vec<ElementData> = serde_json::from_str(data_json).unwrap();

        let mut map = HashMap::new();
        for element in data {
            map.insert(element.symbol.clone(), element);
        }
        map
    };
}

pub fn get_element_data(element_symbol: &str) -> ElementData {
    ELEMENTS
        .get(element_symbol)
        .unwrap_or(&ElementData::default())
        .clone()
}

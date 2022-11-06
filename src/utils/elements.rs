use std::collections::HashMap;

use lazy_static::lazy_static;

#[derive(Debug, serde::Deserialize)]
struct RawElementData {
    symbol: String,
    vdw_radius: f32,
    jmol_color: [f32; 3],
}

lazy_static! {
    static ref ELEMENTS: HashMap<String, RawElementData> = {
        let data_json: &str = include_str!("../../data/elements.json");
        let data: Vec<RawElementData> = serde_json::from_str(data_json).unwrap();

        let mut map = HashMap::new();
        for element in data {
            map.insert(element.symbol.clone(), element);
        }
        map
    };
}

pub struct ElementData {
    pub radius: f32,
    pub color: [f32; 4],
}

impl Default for ElementData {
    fn default() -> Self {
        Self {
            radius: 1.5,
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

pub fn get_element_data(element_symbol: &str) -> ElementData {
    ELEMENTS
        .get(element_symbol)
        .map_or(ElementData::default(), |element| ElementData {
            radius: element.vdw_radius,
            color: [
                element.jmol_color[0],
                element.jmol_color[1],
                element.jmol_color[2],
                1.0,
            ],
        })
}

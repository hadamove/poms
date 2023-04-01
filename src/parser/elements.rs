use lazy_static::lazy_static;
use std::collections::HashMap;

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
    static ref ELEMENTS: HashMap<pdbtbx::Element, ElementData> = {
        // Load element data into program memory during compile time.
        let data_json: &str = include_str!("./data/elements.json");
        let data: Vec<ElementData> = serde_json::from_str(data_json).unwrap();

        let mut map = HashMap::new();
        for element_data in data {
            if let Some(element) = pdbtbx::Element::from_symbol(&element_data.symbol) {
                map.insert(element, element_data);
            }
        }
        map
    };
}

pub fn get_element_data(element: &pdbtbx::Element) -> ElementData {
    ELEMENTS
        .get(element)
        .unwrap_or(&ElementData::default())
        .clone()
}

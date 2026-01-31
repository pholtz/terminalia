use std::collections::HashMap;

use crate::generate::map::Map;

#[derive(Default, Clone)]
pub struct Dungeon {
    maps: HashMap<u32, Map>,
}

impl Dungeon {
    pub fn new() -> Dungeon {
        Dungeon { maps: HashMap::new() }
    }

    pub fn add_map(&mut self, map: &Map) {
        self.maps.insert(map.index, map.clone());
    }

    pub fn get_map(&mut self, index: u32) -> Option<Map> {
        match self.maps.contains_key(&index) {
            true => {
                Some(self.maps[&index].clone())
            }
            false => None
        }
    }
}

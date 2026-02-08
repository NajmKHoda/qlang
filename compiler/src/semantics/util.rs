use std::{collections::HashMap, ops::{Index, IndexMut}};

pub(super) struct IdGenerator {
    next_id: u32,
}

impl IdGenerator {
    pub(super) fn new() -> Self {
        IdGenerator { next_id: 0 }
    }

    pub(super) fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub(super) struct DualLookup<V> {
    name_to_id: HashMap<String, u32>,
    id_to_value: HashMap<u32, V>,
}

impl<V> DualLookup<V> {
    pub(super) fn new() -> Self {
        DualLookup {
            name_to_id: HashMap::new(),
            id_to_value: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, id: u32, value: V) {
        self.name_to_id.insert(name, id);
        self.id_to_value.insert(id, value);
    }

    pub fn get_by_id(&self, id: u32) -> Option<&V> {
        self.id_to_value.get(&id)
    }

    pub fn get_by_id_mut(&mut self, id: u32) -> Option<&mut V> {
        self.id_to_value.get_mut(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&V> {
        match self.name_to_id.get(name) {
            Some(id) => self.id_to_value.get(id),
            None => None,
        }
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut V> {
        match self.name_to_id.get(name) {
            Some(id) => self.id_to_value.get_mut(id),
            None => None,
        }
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    pub fn collect_id_value_map(self) -> HashMap<u32, V> {
        self.id_to_value
    }
}

impl<V> Index<u32> for DualLookup<V> {
    type Output = V;
    fn index(&self, index: u32) -> &Self::Output {
        self.get_by_id(index).unwrap()
    }
}

impl<V> IndexMut<u32> for DualLookup<V> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_by_id_mut(index).unwrap()
    }
}

impl<V> Index<&str> for DualLookup<V> {
    type Output = V;
    fn index(&self, name: &str) -> &Self::Output {
        self.get_by_name(name).unwrap()
    }
}

impl<V> IndexMut<&str> for DualLookup<V> {
    fn index_mut(&mut self, name: &str) -> &mut Self::Output {
        self.get_by_name_mut(name).unwrap()
    }
}

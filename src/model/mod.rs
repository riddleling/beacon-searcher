use gtk::prelude::*;
use std::cell::RefCell;

#[derive(Debug)]
pub struct BeaconInfo {
    pub peripheral_id: String, 
    pub proximity_uuid: String, 
    pub major: u32, 
    pub minor: u32, 
    pub tx_power: i8
}

pub fn create_model() -> gtk::ListStore {
    let types = [
        glib::Type::STRING, 
        glib::Type::STRING, 
        glib::Type::U32, 
        glib::Type::U32, 
        glib::Type::I8
    ];
    let model = gtk::ListStore::new(&types);
    model
}

pub fn add_item(model: &gtk::ListStore, ids: &RefCell<Vec<String>>, beacon: BeaconInfo) {
    if !ids.borrow().contains(&beacon.peripheral_id) {
        ids.borrow_mut().push(beacon.peripheral_id.clone());
        let values: [(u32, &dyn ToValue); 5] = [
            (0, &beacon.peripheral_id),
            (1, &beacon.proximity_uuid),
            (2, &beacon.major),
            (3, &beacon.minor),
            (4, &beacon.tx_power)
        ];
        model.set(&model.append(), &values);
    }
}

pub fn set_tree_view_titles(tree_view: &gtk::TreeView) {
    let titles = vec!["Peripheral ID", "Proximity UUID", "Major", "Minor", "TX Power"];
    for (index, title) in titles.into_iter().enumerate() {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title(title);
        column.add_attribute(&renderer, "text", index as i32);
        column.set_sort_column_id(index as i32);
        tree_view.append_column(&column);
    }
}

pub fn get_row_data_string(model: &gtk::ListStore, path: &gtk::TreePath) -> String {
    let iter = model.iter(path).unwrap();
    let mut string = "".to_string();
    // Peripheral ID
    if let Ok(value) = model.value(&iter,0).get::<String>() {
        string.push_str(&format!("Peripheral ID: {}, \n", value));
    }
    // Proximity UUID
    if let Ok(value) = model.value(&iter,1).get::<String>() {
        string.push_str(&format!("Proximity UUID: {}, \n", value));
    }
    // Major
    if let Ok(value) = model.value(&iter,2).get::<u32>() {
        string.push_str(&format!("Major: {}, \n", value));
    }
    // Minor
    if let Ok(value) = model.value(&iter,3).get::<u32>() {
        string.push_str(&format!("Minor: {}, \n", value));
    }
    // TX Power
    if let Ok(value) = model.value(&iter,4).get::<i8>() {
        string.push_str(&format!("TX Power: {}", value));
    }
    string
}

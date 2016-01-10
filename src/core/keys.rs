use std::collections::HashMap;
use term::terminfo::TermInfo;

lazy_static! {
    static ref KEY_NAME_CONVERSIONS: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("ENT".to_string(), "ENTER".to_string());
        m.insert("IC".to_string(), "INSERT".to_string());
        m.insert("CUB1".to_string(), "LEFT".to_string());
        m.insert("CUD1".to_string(), "DOWN".to_string());
        m.insert("CUF1".to_string(), "RIGHT".to_string());
        m.insert("BS".to_string(), "BACKSPACE".to_string());
        m.insert("NP".to_string(), "PAGEDOWN".to_string());
        m.insert("PP".to_string(), "PAGEUP".to_string());
        m.insert("CBT".to_string(), "BACKTAB".to_string());
        m.insert("DC".to_string(), "DELETE".to_string());
        m
    };
}

pub fn get_key_codes_to_names() -> HashMap<Vec<u8>, String> {
    let mut key_codes_to_names = HashMap::new();
    let info = TermInfo::from_env()
        .expect("Could not load terminfo for current environment");
    for (name, val) in info.strings {
        if name.starts_with("k") {
            let new_name = if name.starts_with("key_") {
                &name[4..]
            } else {
                &name[1..]
            }.to_uppercase();
            key_codes_to_names.insert(val,
                match KEY_NAME_CONVERSIONS.get(&new_name) {
                    Some(n) => n.clone(),
                    None => new_name.clone()
                });
        }
    }

    for i in 0x20..0x7F {
        let name = (i as char).to_string();
        key_codes_to_names.insert(vec![i], name);
    }

    key_codes_to_names
}

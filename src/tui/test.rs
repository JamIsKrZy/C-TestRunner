use std::collections::HashMap;

use maplit::{btreemap, hashmap};

use crate::record_collection::{collection::CompiledRecord, StatusType};


pub fn init_test_collection() -> CompiledRecord {
    CompiledRecord { 
        test_tree: btreemap! {
            "Program1".to_string() => hashmap! {
                "Addtion".to_string() => 0
            }
        }, 
        test_status: vec![
            StatusType::Success
        ], 
        test_logs: vec![
            None
        ] 
    }
}
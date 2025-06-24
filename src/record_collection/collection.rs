use std::{collections::{BTreeMap, HashMap, HashSet}, sync::{Mutex, RwLock}};

use crate::record_collection::{Log, ProcessInfo, Register, Status};
use super::ProgramInfoType;
use super::{LogTypeMessage, RecordErr, StatusType};


type Logs = Vec<LogTypeMessage>;
type TestKeys = HashSet<String>;

struct Test(Status, Mutex<Logs>);

struct TestCollection {
    test_map: BTreeMap<String, TestKeys>,
    test_info: HashMap<String, Test>,
}

pub struct TestRecord(RwLock<TestCollection>);

impl TestRecord {
    pub fn new() -> Self {
        Self(RwLock::new(TestCollection {
            test_map: BTreeMap::new(),
            test_info: HashMap::new(),
        }))
    }

    pub fn process_data(&self, data: ProcessInfo) -> Result<(),()>{
        match data.info_type {
            ProgramInfoType::Register => unsafe{
                println!("{:#?}", data.data.reg);
                // self.register_test(data.data.reg)
            },
            ProgramInfoType::Status => unsafe{
                println!("{:#?}", data.data.stat);
                // self.register_test(data.data.stat)
            },
            ProgramInfoType::Log => unsafe{
                println!("{:#?}", data.data.log);
                // self.register_test(data.data.log)
            },
        }
        Ok(())
    }

    pub fn register_process(&self, process_name: String) -> Result<(), RecordErr> {
        let mut collection = self.0.write().map_err(|_| RecordErr::TestMapCorrupted)?;

        if collection.test_map.contains_key(&process_name) {
            return Err(RecordErr::RecordAlreadyExist);
        } else {
            collection
                .test_map
                .insert(process_name, HashSet::with_capacity(4));
            Ok(())
        }
    }


    fn register_test(&self, test_data: Register) -> Result<(), RecordErr> {
        let mut collection = self.0.write().map_err(|_| RecordErr::TestMapCorrupted)?;

        Ok(())
    }

    fn update_test_status(
        &self,
        stat: Status
    ) -> Result<(), RecordErr> {
        Ok(())
    }

    fn append_test_logs(
        &self, 
        log: Log
    ) {

    }
}

  

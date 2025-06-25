use std::{collections::{BTreeMap, HashMap}, ops::Index, sync::{Arc, Mutex, RwLock}};

use crate::record_collection::{Log, ProcessInfo, Register, Status};
use super::ProgramInfoType;
use super::{LogTypeMessage, RecordErr, StatusType};


type Logs = Vec<LogTypeMessage>;
type TestKeys = RwLock<HashMap<String, usize>>;
type Test = Mutex<(StatusType, Logs)>;


trait StoreData{
    type T;
    type U;

    fn store(&self, data: Self::T) -> Result<(), Self::U>;
}



struct TestCollection {
    test_map: RwLock<BTreeMap<String, TestKeys>>,
    test_list: RwLock<Vec<Test>>,
}

pub struct TestRecord(Arc<TestCollection>);

impl TestRecord {
    pub fn new() -> Self {
        Self(Arc::new(TestCollection {
            test_map: RwLock::new(BTreeMap::new()),
            test_list: RwLock::new(Vec::new()),
        }))
    }

    fn new_test_entry(&self) -> Option<usize>{
        let Ok(mut write_list) = self.0.test_list.write() else {
            return None;
        };

        let index = write_list.len();
        write_list.push(Mutex::new((StatusType::Fail, Vec::new())));
        Some(index)
    }   


    fn register_process(&self, process_name: String) -> Result<(), RecordErr> {
        let Ok(mut c) = self.0.test_map.write() else {
            todo!()
        };

        c.entry(process_name).or_insert(
            RwLock::new(HashMap::new())
        );

        Ok(())
    }


    fn register_test(&self, test_data: Register) -> Result<(), RecordErr> {
        let Ok(c) = &self.0.test_map.read() else {
            todo!()
        };

        let (program_name, function_name) = {
            (
                String::from_utf8(test_data.program_name.to_vec()).map_err(|_| RecordErr::UNDEFINED_ERR)?,
                String::from_utf8(test_data.function_name.to_vec()).map_err(|_| RecordErr::UNDEFINED_ERR)?,
            )
        };

        let mut process_data = c.get(&program_name)
            .ok_or(RecordErr::UNDEFINED_ERR)?
            .write()
            .map_err(|_| RecordErr::UNDEFINED_ERR)?;


        let entry_index = self.new_test_entry()
            .ok_or(RecordErr::UNDEFINED_ERR)?;

        process_data.insert(function_name, entry_index);

        Ok(())
    }

    fn update_test_status(
        &self,
        stat: Status
    ) -> Result<(), RecordErr> {
        let Ok(test_map) = &self.0.test_map.read() else {
            todo!()
        };

        let (program_name, function_name) = {
            (
                String::from_utf8(stat.program_name.to_vec()).map_err(|_| RecordErr::UNDEFINED_ERR)?,
                String::from_utf8(stat.function_name.to_vec()).map_err(|_| RecordErr::UNDEFINED_ERR)?,
            )
        };


        // search through the map
        let test_index = test_map.get(&program_name)
            .ok_or(RecordErr::UNDEFINED_ERR)?
            .read()
            .map_err(|_| RecordErr::UNDEFINED_ERR)?
            .get(&function_name)
            .copied()
            .ok_or(RecordErr::UNDEFINED_ERR)?;

        let Ok(read_list) = self.0.test_list.read() else {
            todo!()
        };

        let mut test_lock = read_list.index(test_index)
            .lock()
            .map_err(|_| RecordErr::UNDEFINED_ERR)?;

        test_lock.0 = stat.t;

        Ok(())
    }

    fn append_test_logs(
        &self, 
        log: Log
    ) -> Result<(), RecordErr> {
        let Ok(test_map) = &self.0.test_map.read() else {
            todo!()
        };

        let (program_name, function_name) = {
            (
                String::from_utf8(log.program_name.to_vec()).map_err(|_| RecordErr::UNDEFINED_ERR)?,
                String::from_utf8(log.function_name.to_vec()).map_err(|_| RecordErr::UNDEFINED_ERR)?,
            )
        };

        let test_index = test_map.get(&program_name)
            .ok_or(RecordErr::UNDEFINED_ERR)?
            .read()
            .map_err(|_| RecordErr::UNDEFINED_ERR)?
            .get(&function_name)
            .copied()
            .ok_or(RecordErr::UNDEFINED_ERR)?;

        let Ok(read_list) = self.0.test_list.read() else {
            todo!()
        };

        let mut test_lock = read_list.index(test_index)
            .lock()
            .map_err(|_| RecordErr::UNDEFINED_ERR)?;
        
        test_lock.1.push(log.into());

        Ok(())
    }
}


impl Clone for TestRecord{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
} 

impl StoreData for TestRecord{
    type T = ProcessInfo;
    type U = ();

    fn store(&self, data: Self::T) -> Result<(), Self::U> {
        match data.info_type {
            ProgramInfoType::Register => unsafe{
                println!("{}", data.data.reg);
                // self.register_test(data.data.reg)
            },
            ProgramInfoType::Status => unsafe{
                println!("{}", data.data.stat);
                // self.register_test(data.data.stat)
            },
            ProgramInfoType::Log => unsafe{
                println!("{}", data.data.log);
                // self.register_test(data.data.log)
            },
        }
        Ok(())
    }
}
  

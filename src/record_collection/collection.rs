use std::{
    collections::{BTreeMap, HashMap},
    ops::Index,
    sync::{Arc, Mutex, RwLock},
};

use termion::color;

use super::ProgramInfoType;
use super::{LogTypeMessage, RecordErr, StatusType};
use crate::{
    record_collection::{Log, ProcessInfo, Register, Status},
    util,
};

type Logs = Vec<LogTypeMessage>;
type TestKeys = RwLock<HashMap<String, usize>>;
type Test = Mutex<(StatusType, Logs)>;

pub trait StoreData {
    type T;
    type U;

    fn store(&self, data: Self::T) -> Result<(), Self::U>;
}

#[derive(Debug)]
struct TestCollection {
    test_map: RwLock<BTreeMap<String, TestKeys>>,
    test_list: RwLock<Vec<Test>>,
}

#[derive(Debug)]
pub struct TestRecord(Arc<TestCollection>);

#[derive(Debug)]
pub struct CompiledRecord{
    test_tree: BTreeMap<String, HashMap<String, usize>>,
    test_list: Vec<(StatusType, Vec<LogTypeMessage>)>
}

impl TestRecord {
    pub fn new() -> Self {
        Self(Arc::new(TestCollection {
            test_map: RwLock::new(BTreeMap::new()),
            test_list: RwLock::new(Vec::new()),
        }))
    }

    pub fn compile(self) -> Result<CompiledRecord, ()> {
        let s = Arc::into_inner(self.0)
            .ok_or(())?;
        
        let inner_tree = s.test_map.into_inner()
            .map_err(|_| ())?
            .into_iter()
            .map(|(k, rwlock_map)| -> Result<_, ()> {
                let v = rwlock_map.into_inner()
                    .map_err(|_| ())?;
                    
                Ok((k,v))
            })
            .filter_map(Result::ok)
            .collect::<BTreeMap<_,_>>();


        let inner_list = s.test_list.into_inner()
            .map_err(|_| ())?
            .into_iter()
            .map(Mutex::into_inner)
            .filter_map(Result::ok)
            .collect::<Vec<_>>();


        Ok(CompiledRecord { 
            test_tree: inner_tree, 
            test_list: inner_list 
        })

    }

    fn new_test_entry(&self) -> Option<usize> {
        let Ok(mut write_list) = self.0.test_list.write() else {
            return None;
        };

        let index = write_list.len();
        write_list.push(Mutex::new((StatusType::Fail, Vec::new())));
        Some(index)
    }

    pub fn register_process(&mut self, process_name: String) -> Result<(), RecordErr> {
        let mut c = self
            .0
            .test_map
            .write()
            .map_err(|_| RecordErr::PoisonedWrite)?;

        c.entry(process_name).or_insert(RwLock::new(HashMap::new()));

        Ok(())
    }

    fn register_test(&self, test_data: Register) -> Result<(), RecordErr> {
        let c = &self
            .0
            .test_map
            .read()
            .map_err(|_| RecordErr::PoisonedWrite)?;

        let (program_name, function_name) = {
            (
                util::bytes_to_trimmed_string(&test_data.program_name)
                    .map_err(|_| RecordErr::Utf8ConvertionErr)?,
                util::bytes_to_trimmed_string(&test_data.function_name)
                    .map_err(|_| RecordErr::Utf8ConvertionErr)?,
            )
        };

        let mut process_data = c
            .get(program_name.trim())
            .ok_or(RecordErr::ProgramNotExist)?
            .write()
            .map_err(|_| RecordErr::PoisonedWrite)?;

        let entry_index = self.new_test_entry().ok_or(RecordErr::PoisonedLock)?;

        process_data.insert(function_name, entry_index);

        Ok(())
    }

    fn update_test_status(&self, stat: Status) -> Result<(), RecordErr> {
        let test_map = &self
            .0
            .test_map
            .read()
            .map_err(|_| RecordErr::PoisonedWrite)?;

        let (program_name, function_name) = {
            (
                util::bytes_to_trimmed_string(&stat.program_name)
                    .map_err(|_| RecordErr::Utf8ConvertionErr)?,
                util::bytes_to_trimmed_string(&stat.function_name)
                    .map_err(|_| RecordErr::Utf8ConvertionErr)?,
            )
        };

        // search through the map
        let test_index = test_map
            .get(&program_name)
            .ok_or(RecordErr::ProgramNotExist)?
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?
            .get(&function_name)
            .copied()
            .ok_or(RecordErr::TestNotExist)?;

        let read_list = self
            .0
            .test_list
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?;

        let mut test_lock = read_list
            .index(test_index)
            .lock()
            .map_err(|_| RecordErr::PoisonedLock)?;

        test_lock.0 = stat.t;

        Ok(())
    }

    fn append_test_logs(&self, log: Log) -> Result<(), RecordErr> {
        let test_map = &self
            .0
            .test_map
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?;

        let (program_name, function_name) = {
            (
                util::bytes_to_trimmed_string(&log.program_name)
                    .map_err(|_| RecordErr::Utf8ConvertionErr)?,
                util::bytes_to_trimmed_string(&log.function_name)
                    .map_err(|_| RecordErr::Utf8ConvertionErr)?,
            )
        };

        let test_index = test_map
            .get(&program_name)
            .ok_or(RecordErr::ProgramNotExist)?
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?
            .get(&function_name)
            .copied()
            .ok_or(RecordErr::TestNotExist)?;

        let read_list = self
            .0
            .test_list
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?;

        let mut test_lock = read_list
            .index(test_index)
            .lock()
            .map_err(|_| RecordErr::PoisonedLock)?;

        test_lock.1.push(log.into());

        Ok(())
    }
}

impl Clone for TestRecord {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl StoreData for TestRecord {
    type T = ProcessInfo;
    type U = ();

    fn store(&self, data: Self::T) -> Result<(), Self::U> {
        let i = match data.info_type {
            ProgramInfoType::Register => unsafe {
                self.register_test(data.data.reg)
                // self.register_test(data.data.reg)
            },
            ProgramInfoType::Status => unsafe {
                self.update_test_status(data.data.stat)
                // self.register_test(data.data.stat)
            },
            ProgramInfoType::Log => unsafe {
                self.append_test_logs(data.data.log)
                // self.register_test(data.data.log)
            },
        };

        if let Err(it) = i {
            println!(
                "{}FAIL: {:?}{}\n{}",
                termion::color::Fg(color::Red),
                it,
                termion::color::Fg(color::Reset),
                data
            );
        }

        Ok(())
    }
}

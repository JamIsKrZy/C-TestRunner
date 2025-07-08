use std::{
    cell::{RefCell, UnsafeCell}, collections::{ BTreeMap, HashMap}, fmt::Display, ops::{Index, IndexMut, Range}, sync::{Arc, Mutex, RwLock}
};

use const_format::concatcp;
use termion::color;

use super::ProgramInfoType;
use super::{LogTypeMessage, RecordErr, StatusType};
use crate::{
    record_collection::{Log, ProcessInfo, Register, Status},
    util,
};

type Logs = Vec<LogTypeMessage>;
type TestKeys = RwLock<HashMap<String, usize>>;

pub trait StoreData {
    type T;
    type U;

    fn store(&self, data: Self::T) -> Result<(), Self::U>;
}


#[derive(Debug)]
struct TestCollection {
    test_map: RwLock<BTreeMap<String, TestKeys>>,
    test_status: Mutex<Vec<StatusType>>,
    test_logs: RwLock<Vec<Mutex<Option<Logs>>>>
}


#[derive(Debug)]
pub struct TestRecord(Arc<TestCollection>);

#[derive(Debug)]
pub(crate) struct CompiledRecord{
    pub(crate) test_tree: BTreeMap<String, HashMap<String, usize>>,
    pub(crate) test_status: Vec<StatusType>,
    pub(crate) test_logs: Vec<Option<Vec<LogTypeMessage>>>
}

#[derive(Debug, Clone)]
pub(crate) struct SummarizedPrograms<'a>{
    programs: Vec<[&'a str; 2]>
}

pub(crate) struct SummarizedTest<'a>{
    tests: Vec<Vec<[&'a str;2]>>
}

impl TestRecord {
    pub fn new() -> Self {
        Self(Arc::new(TestCollection {
            test_map: RwLock::new(BTreeMap::new()),
            test_status: Mutex::new(Vec::new()),
            test_logs: RwLock::new(Vec::new()),
        }))
    }

    pub fn compile(self) -> Result<CompiledRecord, ()> {
        let s = Arc::into_inner(self.0)
            .ok_or(())?;

        let test_tree = s.test_map.into_inner()
            .map_err(|_| ())?
            .into_iter()
            .map(|(k, rwlock_map)| -> Result<_, ()> {
                let v = rwlock_map.into_inner()
                    .map_err(|_| ())?;

                Ok((k,v))
            })
            .filter_map(Result::ok)
            .collect();


        let test_status = s.test_status.into_inner()
            .map_err(|_| ())?
            .into_iter()
            .collect();


        let test_logs = s.test_logs.into_inner()
            .map_err(|_| ())?
            .into_iter()
            .map(Mutex::into_inner)
            .filter_map(Result::ok)
            .collect();


        Ok(CompiledRecord { 
            test_tree,
            test_status, 
            test_logs 
        })

    }

        fn new_test_entry(&self) -> Option<usize> {
        let Ok(mut write_list_stat) = self.0.test_status.lock() else {
            return None;
        };

        let index = write_list_stat.len();

        write_list_stat.push(StatusType::Fail);
        drop(write_list_stat);

        let Ok(mut write_list_logs) = self.0.test_logs.write() else {
            return None;
        };

        write_list_logs.push(Mutex::new(None));


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

        // search through the map to get the index of test
        let test_index = test_map
            .get(&program_name)
            .ok_or(RecordErr::ProgramNotExist)?
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?
            .get(&function_name)
            .copied()
            .ok_or(RecordErr::TestNotExist)?;

        let mut mutex_vec = self
            .0
            .test_status
            .lock()
            .map_err(|_| RecordErr::PoisonedRead)?;

        let test_ref = mutex_vec.index_mut(test_index);

        *test_ref = stat.t;

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

        self.0
            .test_logs
            .read()
            .map_err(|_| RecordErr::PoisonedRead)?
            .index(test_index)
            .lock()
            .map(|mut m|{

                if let Some(v) = m.as_mut() {
                    v.push(log.into());
                } else {
                    *m = Some(vec![log.into()])
                }
 
            })
            .map_err(|_| RecordErr::PoisonedLock)?;
            
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


impl<'a> Index<usize> for SummarizedTest<'a>{
    type Output = Vec<[&'a str; 2]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tests[index]
    }
}



const SUCCESS_STR: &str =  "SUCCESS";
const FAIL_STR: &str = "FAIL";


impl CompiledRecord {

    ///
    /// this is function is unsafe, first index of the tuple
    /// must not be mutated
    pub fn summarize<'a>(&'a self) -> (Vec<UnsafeCell<String>>, SummarizedPrograms<'a>, SummarizedTest<'a>){
        let mut owned_string_format = Vec::new();
        let mut programs = Vec::new();
        let mut program_details = Vec::new();

        for (program, tests) in self.test_tree.iter() {
            let total_test= tests.len();
            let mut total_success = 0usize;

            let mut test_list = Vec::new();
            
            for (test_name, &index_ref) in tests{
                let test_info;
                if self.test_status[index_ref] == StatusType::Success {
                    test_info = [test_name.as_str(), SUCCESS_STR];
                    total_success += 1;

                } else {
                    test_info = [test_name.as_str(), FAIL_STR];
                }
                test_list.push(test_info);

            }

            let format_total = format!("   {}/{}", total_success, total_test);

            let borrowed_result = {
                owned_string_format.push(UnsafeCell::new(format_total));

                unsafe {
                    owned_string_format.last()
                        .expect("Owned string is empty")
                        .get()
                        .as_ref()
                        .expect("invalid as ref")
                        .as_str()
                }
            };


            program_details.push(test_list);
            programs.push([program.as_str(), borrowed_result]);

        }

        (
            owned_string_format,
            SummarizedPrograms { programs },
            SummarizedTest { tests: program_details }
        )
    }

 
}

impl<'a> Index<usize> for SummarizedPrograms<'a>{
    type Output = [&'a str; 2];

    fn index(&self, index: usize) -> &Self::Output {
        &self.programs[index]
    }
}


impl Display for CompiledRecord{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        writeln!(f,"CompiledRecord {{")?;

        for i in self.test_tree.iter(){
            writeln!(f,"\t\"{}\"{{", i.0)?;

            for (test_name, &index) in i.1.iter(){
                let status = self.test_status[index];
                let log_count = self.test_logs
                    .index(index)
                    .as_ref()
                    .map(Vec::len)
                    .unwrap_or(0);

                writeln!(f,"\t\t\"{}\":\tStatus: {:?}\tLogs_count: {}", 
                    test_name,
                    status,
                    log_count 
                )?;
            }
            writeln!(f,"\t}},")?;
        }



        writeln!(f,"}}")
    }
}

impl SummarizedPrograms<'_>{
    
    pub fn len(&self) -> usize{
        self.programs.len()
    }
}

impl<'a> Index<Range<usize>> for SummarizedPrograms<'a>{
    type Output = [[&'a str; 2]];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.programs[index]
    }
} 
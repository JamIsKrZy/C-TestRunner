use std::{fmt::Display, io};


pub mod collection;

enum RecordErr {
    TestMapCorrupted,
    TestInfoCorrupted,
    RecordAlreadyExist,
    StatusIsDefined,
    UNDEFINED_ERR
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
enum StatusType {
    Success,
    Fail,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Status{
    program_name: [u8;32],
    function_name: [u8;32],
    t: StatusType,
}

impl Display for Status{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let program_name = String::from_utf8_lossy(&self.program_name);
        let function_name = String::from_utf8_lossy(&self.function_name);

        write!(f, "{{ \n\tstatus: {:?}\n\tprogram_name: {},\n\tfunction_name: {}\n}}",
            self.t,
            program_name,
            function_name
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Register{
    program_name: [u8;32],
    function_name: [u8;32],
}

impl Display for Register{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let program_name = String::from_utf8_lossy(&self.program_name);
        let func_name = String::from_utf8_lossy(&self.function_name);
        
        write!(f, "{{ \n\tprogram_name: {}, \n\tfunction_name: {}\n}}", 
            program_name,
            func_name
        )
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
enum LogType{
    Debug,
    Info,
    Warning
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Log{
    program_name: [u8;32],
    function_name: [u8;32],
    msg: [u8;64],
    t: LogType
}

impl Display for Log{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let program_name = String::from_utf8_lossy(&self.program_name);
        let func_name = String::from_utf8_lossy(&self.function_name);
        let msg = String::from_utf8_lossy(&self.msg);

        write!(f, "{{ \n\tLogType: {:?},\n\tprogram_name: {},\n\tfunction_name: {},\n\tmsg: {} \n}}",
            self.t,
            program_name,
            func_name,
            msg,
        )
    }
}

#[repr(C)]
enum ProgramInfoType {
    Register = 0,
    Status = 1,
    Log = 2
}

#[repr(C)]
pub union ProgramData {
    pub log: Log,
    pub reg: Register,
    pub stat: Status,
}

#[repr(C)]
pub struct ProcessInfo {
    data: ProgramData,
    info_type: ProgramInfoType,
}

impl Display for ProcessInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.info_type {
            ProgramInfoType::Register => unsafe {
                write!(f, "[Register]{}", self.data.reg)
            },
            ProgramInfoType::Status => unsafe {
                write!(f, "[Status]{}", self.data.stat)
            },
            ProgramInfoType::Log => unsafe {
                write!(f, "[Log]{}", self.data.log)
            },
        }
    }
}


enum LogTypeMessage {
    Debug(String),
    Info(String),
    Warning(String),
}

impl From<Log> for LogTypeMessage{
    fn from(value: Log) -> Self {
        match value.t {
            LogType::Debug => LogTypeMessage::Debug(
                String::from_utf8(value.msg.to_vec())
                    .unwrap_or("[Data Courrpted]".to_string())
            ),
            LogType::Info => LogTypeMessage::Info(
                String::from_utf8(value.msg.to_vec())
                    .unwrap_or("[Data Courrpted]".to_string())
            ),
            LogType::Warning => LogTypeMessage::Warning(
                String::from_utf8(value.msg.to_vec())
                    .unwrap_or("[Data Courrpted]".to_string())
            ),
        }
    }
}

#[inline(always)]
pub fn bin_convert(refe: &[u8; std::mem::size_of::<ProcessInfo>()]) -> ProcessInfo {
    unsafe {
        std::ptr::read(refe.as_ptr() as *const ProcessInfo)
    }
}





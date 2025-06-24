use std::{fmt::Display, io};


pub mod collection;

enum RecordErr {
    TestMapCorrupted,
    TestInfoCorrupted,
    RecordAlreadyExist,
    StatusIsDefined,
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
    file_name: [u8;32],
    function_name: [u8;32],
    t: StatusType,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Register{
    file_name: [u8;32],
    function_name: [u8;32],
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

        write!(f, "{{ LogType: {:?},\nprogram_name: {},\nfunction_name: {},\nmsg: {} }}",
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
                write!(f, "[Register]{:?}", self.data.reg)
            },
            ProgramInfoType::Status => unsafe {
                write!(f, "[Status]{:?}", self.data.stat)
            },
            ProgramInfoType::Log => unsafe {
                write!(f, "[Log]{:?}", self.data.log)
            },
        }
    }
}


enum LogTypeMessage {
    Debug(String),
    Info(String),
    Warning(String),
}

#[inline(always)]
pub fn bin_convert(refe: &[u8; std::mem::size_of::<ProcessInfo>()]) -> ProcessInfo {
    unsafe {
        std::ptr::read(refe.as_ptr() as *const ProcessInfo)
    }
}





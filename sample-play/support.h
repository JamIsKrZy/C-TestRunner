#ifndef RUNTIME_SUPPORT
#define RUNTIME_SUPPORT


extern const char* PROGRAM_NAME;


/**
 * Process communication to send data to 
 * assigned pipe OR file descriptor/handler
 */

#define MESSAGE_BUFFER 64
#define FUNCTION_MAX_CHAR_SIZE 32
#define PROGRAM_NAME_MAX_CHAR_SIZE 64


// inform runtime to register testcase
// register-{program_name}-{func_name}
// program_name - passed from args
// register - signal type
// func_name - user defined


// inform runtime to set status test case finished
// status-{status}-{program_name}-{func_name}
// program_name - passed from args
// func_name - registered func
// status - signal type
// status - status of the test case


// send log
// log-{type}-{program_name}-{func_name}-{msg}

enum StatusType{
    Success,
    Fail 
};

struct Status{
    char program_name[PROGRAM_NAME_MAX_CHAR_SIZE];
    char function_name[FUNCTION_MAX_CHAR_SIZE];
    enum StatusType t;
};

struct Register{
    char program_name[PROGRAM_NAME_MAX_CHAR_SIZE];
    char function_name[FUNCTION_MAX_CHAR_SIZE];
};

enum LogType{
    Debug,
    Info, 
    Warning
};

struct Log{
    char program_name[PROGRAM_NAME_MAX_CHAR_SIZE];
    char function_name[FUNCTION_MAX_CHAR_SIZE];
    char msg[MESSAGE_BUFFER];
    enum LogType t;
};



enum ProgramInfoType {
    Register = 0,
    Status = 1,
    Log = 2
};


typedef struct ProcessData{
    union {
        struct Log log;
        struct Register reg;
        struct Status stat;
    };
    enum ProgramInfoType info_type;
} ProcessData;


#endif
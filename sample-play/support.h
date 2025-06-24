#ifndef RUNTIME_SUPPORT
#define RUNTIME_SUPPORT

// ---------------------------------------------
//                  Later work on
// ---------------------------------------------

/**
 * Process communication to send data to 
 * assigned pipe OR file descriptor/handler
 */

#define MESSAGE_BUFFER 64
#define FUNCTION_MAX_CHAR_SIZE 32

enum StatusType{
    Success,
    Fail 
};

struct Status{
    char file_name[32];
    char function_name[FUNCTION_MAX_CHAR_SIZE];
    enum StatusType t;
};

struct Register{
    char file_name[32];
    char function_name[FUNCTION_MAX_CHAR_SIZE];
};

enum LogType{
    Debug,
    Info, 
    Warning
};

struct Log{
    char file_name[32];
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
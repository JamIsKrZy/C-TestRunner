#ifndef ESSENTIAL_TESTING_HEADER
#define ESSENTIAL_TESTING_HEADER

#include <pthread.h>
#include <stdlib.h>
#include "support.h"

#define MB_1 (1024 * 1024)
#define KB_1 (1024)
#define DEFAULT_SSIZE MB_1
#define RETURN_SUCCESS pthread_exit((void*)0)

#define assert(condition) do{\
    if(!(condition)){\
        char* fail_msg = malloc(128); \
        snprintf(fail_msg, 128, "Assertion failed at %s:%d", __FILE__, __LINE__); \
        pthread_exit((void*)fail_msg);\
    }\
} while (0)

#define debug(fmt, ...) do{\
    ProcessData data = {\
        .info_type = Log,\
        .log = {\
            .t = Debug,\
            .msg = {0},\
            .program_name = __FILE__,\
            .function_name = {0}\
        }\
    };\
    snprintf(data.log.msg, MESSAGE_BUFFER, fmt, ##__VA_ARGS__);\
    snprintf(data.log.function_name, FUNCTION_MAX_CHAR_SIZE, "%s", __func__);\
    snprintf(data.log.program_name, PROGRAM_NAME_MAX_CHAR_SIZE, "%s", PROGRAM_NAME);\
    \
    fwrite(\
        &data,\
        1,\
        sizeof(ProcessData),\
        stdout\
    );\
} while(0);


#define TEST(title) void* title(void*)


#endif
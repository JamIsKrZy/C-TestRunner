


#ifdef TEST_CASES

#ifndef INCLUDES_FOR_TEST_RUNTIME
#define INCLUDES_FOR_TEST_RUNTIME
#include <stdio.h>
#include "essential.h"
#include "support.h"
#include <pthread.h>
#include <errno.h>
#endif




const char *PROGRAM_NAME = NULL;







#define TEST_CASE(func, _ssize) void* func(void*);
TEST_CASES
#undef TEST_CASE

struct test_case{
    pthread_t tr;
    size_t ssize;
    char *thread_name;
    void* (*func_ptr)(void*);
};

struct contents {
    struct test_case *c;
    size_t len;
};

struct contents init_thread_contents(){
    void* (*func_ptrs[])(void*) = {
        #define TEST_CASE(func, ssize) func, 
        TEST_CASES
        #undef TEST_CASE
    };

    // store all function namee as arr of strings
    char *function_names[] = {
        #define TEST_CASE(func, _ssize) #func ,
        TEST_CASES
        #undef TEST_CASE
    };
    size_t len = sizeof(function_names)/sizeof(function_names[0]);

    // store in arr all stack size
    size_t ssizes[] = {
        #define TEST_CASE(_func, ssize) ssize ,
        TEST_CASES
        #undef TEST_CASE
    };

    struct test_case *test_list = malloc(sizeof(struct test_case) * len);
    for (size_t i = 0; i < len; i++){
        test_list[i].func_ptr = func_ptrs[i];
        test_list[i].thread_name = function_names[i];
        test_list[i].ssize = ssizes[i]; 
    }
    
    return (struct contents){
        .len = len, .c = test_list
    };
}   






void send_warning_msg(
    const char* program_name, 
    const char* function_name, 
    const char* msg
);
void send_status(
    const char* program_name, 
    char* from_test, 
    enum StatusType t
);
void send_register(
    const char* program_name, 
    char* test_name
);

int main(int argc, char const *argv[]){

    // init global variable
    if (argc > 0) {
        PROGRAM_NAME = argv[0];  
    }

    // const char const *assigned_key = argv[0];
    size_t len; 
    struct test_case *thread_list;
    
    {
        struct contents c = init_thread_contents();
        len = c.len;
        thread_list = c.c;
    }
    
    // init threads
    {
        pthread_attr_t attr;
        pthread_attr_init(&attr);
        // char **results = calloc(len, sizeof(char*));
        for (size_t i = 0; i < len; i++){
            if(thread_list[i].func_ptr == NULL){
                continue;
            }
            
            // send information of test case 
            // to parent test runner process 
            send_register(argv[0], thread_list[i].thread_name);

            pthread_attr_setstacksize(&attr, thread_list[i].ssize);
            
            pthread_create(
                &thread_list[i].tr,
                &attr,
                thread_list[i].func_ptr,
                NULL
            );
        }
        pthread_attr_destroy(&attr);
    }
    
    // wait for results
    size_t waiting = len;
    char *catch;
    while (waiting){
        // Waiting for test case...
        for (size_t i = 0; i < len; i++){

            int res = pthread_tryjoin_np(
                thread_list[i].tr,
                (void**)&catch
            );

            if (res == 0) {

                if(catch){
                    // encounters an error
                    send_status(argv[0], thread_list[i].thread_name, Fail);
                    send_warning_msg(argv[0], thread_list[i].thread_name,(const char*)catch);
                    free(catch);
                } else {

                    // test successfully ended 
                    send_status(argv[0], thread_list[i].thread_name, Success);
                    
                }

                waiting--;
                break;
            } else if (res == EBUSY) {

                // printf("Thread still working...\n");
                // check again after some time
            } else {

                // TODO 
                perror("%s: pthread_tryjoin_np error - TODO!!!");
                break;
            }
            
        }
        
    }

    free(thread_list);

    return 0;
}




void send_status(const char* program_name, char* from_test, enum StatusType t) {
    ProcessData data = {
        .info_type = Status,
        .stat = {
            .program_name = {0},
            .function_name = {0},
            .t = t
        }
    };

    snprintf(
        data.stat.program_name, 
        PROGRAM_NAME_MAX_CHAR_SIZE,
        "%s", 
        program_name
    );
    
    snprintf(
        data.stat.function_name, 
        FUNCTION_MAX_CHAR_SIZE,
        "%s", 
        from_test
    );

    #ifdef __DEBUG__RUNTIME__
        if ((size_t)data.stat.function_name == 0){
            perror("Function name is undefined!");
            break;
        }
    #endif
    

    fwrite(
        &data,
        1,
        sizeof(ProcessData),
        stdout
    );
}

void send_register(const char* program_name, char* test_name){
    ProcessData data = {
        .info_type = Register,
        .reg = {
            .program_name = {0},
            .function_name = {0}
        }
    };

    snprintf(
        data.reg.program_name, 
        PROGRAM_NAME_MAX_CHAR_SIZE,
        "%s", 
        program_name
    );

    snprintf(
        data.stat.function_name, 
        FUNCTION_MAX_CHAR_SIZE,
        "%s", 
        test_name
    );

    fwrite(
        &data,
        1,
        sizeof(ProcessData),
        stdout
    );
}

void send_warning_msg(
    const char* program_name, 
    const char* function_name, 
    const char* msg
){
    ProcessData data = {
        .info_type = Log,
        .log = {\
            .t = Warning,\
            .msg = {0},\
            .program_name = __FILE__,
            .function_name = {0}\
        }
    };

    snprintf(data.log.msg, MESSAGE_BUFFER, "%s", msg);
    snprintf(data.log.function_name, FUNCTION_MAX_CHAR_SIZE, "%s", function_name);
    snprintf(data.log.program_name, PROGRAM_NAME_MAX_CHAR_SIZE, "%s", program_name);
    
    fwrite(
        &data,
        1,
        sizeof(ProcessData),
        stdout
    );
}









#undef TEST_CASES
#else
#error "Undefined Test cases"

#endif




#define TEST_CASES\
    TEST_CASE(addition, DEFAULT_SSIZE)\
    TEST_CASE(multi, DEFAULT_SSIZE)\
    TEST_CASE(emtpy, DEFAULT_SSIZE)\
    TEST_CASE(computey, DEFAULT_SSIZE)\

#include "runtime.h"


TEST(addition){
    int some = 1+1;

    debug("Hello %d", some);


    RETURN_SUCCESS;
}

TEST(multi){
    int some = 1*5;

    RETURN_SUCCESS;
}

TEST(emtpy){
    int some = 0;
    

    RETURN_FAIL;
}

TEST(computey){
    int some[] = {1,2,3,4,5};
    int len = sizeof(some)/sizeof(some[0]);

    int sum = 0;
    for (int i = 0; i < len; i++){
        sum += some[i];
    }

    debug("%d", sum);


    RETURN_SUCCESS;
}
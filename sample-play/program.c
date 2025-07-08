

#define TEST_CASES\
    TEST_CASE(addition, DEFAULT_SSIZE)\
    TEST_CASE(emtpy, DEFAULT_SSIZE)\
    TEST_CASE(computey, DEFAULT_SSIZE)\
    TEST_CASE(array, DEFAULT_SSIZE)

#include "runtime.h"


TEST(array){


    int arr[3] = {0,1,0};

    for (size_t i = 0; i < 3; i++)
    {
        if (arr[i] == 1) {

            debug("%d", 10);
            RETURN_FAIL;
        }
    }
    

    RETURN_SUCCESS;
}



TEST(addition){
    int some = 1+1;

    debug("Hello %d", some);


    RETURN_FAIL;
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


    RETURN_FAIL;
}
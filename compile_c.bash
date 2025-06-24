
O_Name=$1

gcc -o bin_test/testify/$O_Name \
    bin_test/program.c \
    -D_GNU_SOURCE \
    -pthread \
    -Wall -Wextra \
    -fsanitize=address -fsanitize=undefined

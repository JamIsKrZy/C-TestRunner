
Name=$1

gcc sample-play/program.c\
    -o sample-play/test/"$Name"\
    -lpthread -D_GNU_SOURCE

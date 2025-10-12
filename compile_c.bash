Name=$1

gcc sample-play/program.c -o sample-play/testbin/"$Name" \
  -lpthread -D_GNU_SOURCE

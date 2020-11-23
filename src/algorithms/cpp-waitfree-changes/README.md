
# Disclaimer

This modified main.cpp is mainly from repo:
https://github.com/c650/waitfree-vector

It has just been modified in some areas to match the Rust runner to have an equal comparison between the two implementations

# How to
Replace the main.cpp in ./waitfree-vector/src/concurrent/ with this one to use

You can run then "make concurrent" from ./waitfree-vector to compile

Or you can copy over the Dockerfile from here to ./waitfree-vector and run
docker build . -t waitfree-cpp
docker run --rm waitfree-cpp

NOTE: Docker perfomance is slower for this benchmark than running native

FROM frolvlad/alpine-gxx as builder
WORKDIR /usr/src/concurrent
COPY . . 
RUN apk add --no-cache make
RUN make concurrent
CMD ["/usr/src/concurrent/bin/concurrent.out"]



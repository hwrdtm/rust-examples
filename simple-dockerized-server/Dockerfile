# Base image extends rust:nightly which extends debian:buster-slim
FROM rustlang/rust:nightly as build

# Copy the source code
RUN mkdir -p /someworkdir
COPY Cargo.* /someworkdir
COPY src/ /someworkdir/src/

# Set the working directory
WORKDIR /someworkdir    

CMD ["cargo", "run"]
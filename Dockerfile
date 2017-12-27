FROM rust:1.22.1-jessie

LABEL maintainer="john@jgeer.com"

WORKDIR /home/

# Using the rusty-blockparser package
# https://github.com/gcarq/rusty-blockparser#Usage

# Copy and build the program
COPY rusty-blockparser .
RUN cargo build --release
RUN cargo test --release
# Could also just pull it from Cargo
# RUN cargo install rusty-blockparser

CMD ./target/release/rusty-blockparser \
    --coin bitcoin --threads 4 \
    --chain-storage /home/csv-data/chain.json \
    --blockchain-dir /home/bitcoin/ \
    csvdump \
    /home/csv-data/

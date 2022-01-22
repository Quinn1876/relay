FROM rust

COPY . /relay

WORKDIR /relay

CMD ["cargo run"]

FROM rust as builder
ADD . /root
RUN cd /root && cargo build --release

FROM rust
COPY --from=builder /root/target/release/ord /usr/local/bin/ord


FROM kentsday/dfx_rust:bullseye-dfx-0.10.1-1
COPY config.toml /usr/local/cargo/

RUN rustup self uninstall -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN rustup target add wasm32-unknown-unknown
RUN dfx upgrade
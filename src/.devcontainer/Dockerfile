FROM kentsday/dfx_rust:bullseye-dfx-0.10.1-1
COPY config.toml /usr/local/cargo/

RUN rustup self uninstall -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN DFX_VERSION=0.15.1 sh -ci "$(curl -fsSL https://raw.githubusercontent.com/dfinity/sdk/dfxvm-install-script/install.sh)"
RUN dfx upgrade
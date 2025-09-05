# https://just.systems
set windows-shell := ["C:\\Program Files\\Git\\bin\\sh.exe","-c"]

dev:
    cargo watch -w src -x run -c
build:
    cargo build --release
build-gnu:
    cross build --target x86_64-unknown-linux-gnu --release
build-musl:
    cross build --target x86_64-unknown-linux-musl --release

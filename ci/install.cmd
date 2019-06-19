curl -sSf -o rustup-init.exe https://win.rustup.rs/
rustup-init.exe -y --default-host %TARGET% --default-toolchain %TRAVIS_RUST_VERSION%

set PATH=%PATH%;C:\Users\travis\.cargo\bin
rustc -Vv
cargo -V

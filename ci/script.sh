# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    local cargo=cargo
    if [ $TRAVIS_OS_NAME = linux ] || [ $TRAVIS_OS_NAME = osx ]; then
      cargo=cross
    fi

    case $TARGET in
        x86_64-pc-windows-gnu)
            export PATH=$PATH:C:\msys64\mingw64\bin
            ;;
        i686-pc-windows-gnu)
            export PATH=$PATH:C:\msys64\mingw32\bin
            ;;
    esac

    $cargo build --target $TARGET
    $cargo build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    $cargo test --target $TARGET
    $cargo test --target $TARGET --release

    $cargo run --target $TARGET --bin uvm -- --help
    $cargo run --target $TARGET --bin uvm --release -- --help
 }

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi

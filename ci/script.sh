# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    local cargo=cargo
    if [ $TRAVIS_OS_NAME = linux ] || [ $TRAVIS_OS_NAME = osx ]; then
      cargo=cross
    fi

    $cargo build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    if [ ! -z $INSTALLER_TESTS ]; then
      $cargo test installs_editor_and_modules_ --target $TARGET --release -- --nocapture --ignored
    else
      $cargo test --target $TARGET --release
      $cargo run --target $TARGET --bin uvm --release -- --help
    fi
 }

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi

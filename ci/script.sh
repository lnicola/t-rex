# This script takes care of testing your crate

set -ex

main() {
    if [ ! -z $DISABLE_TESTS ]; then
        cargo build --target $TARGET
        cargo build --target $TARGET --release
        return
    fi

    #cargo test --target $TARGET
    # cross failes with linking -lgdal
    cargo test --all --target $TARGET --release

    cargo test --all --target $TARGET
    # libgdal-dev from ubuntugis drops postgresql-9.4-postgis-2.3
    if [ $TRAVIS_OS_NAME = osx ]; then
        # cross ignores DBCONN env variable (https://github.com/japaric/cross/issues/76)
        cargo test --all --target $TARGET -- --ignored
    fi

    #cargo run --target $TARGET
    cargo run --target $TARGET --release

    if [ $TRAVIS_OS_NAME = linux ]; then
        ldd target/$TARGET/release/t_rex
    fi
    if [ $TRAVIS_OS_NAME = osx ]; then
        otool -L target/$TARGET/release/t_rex
    fi
}

dockerimg() {
    pushd packaging/docker
    # travis uid/gid is 2000/2000, but we build with default uid 1000
    docker build -t t-rex-tileserver/t-rex -f Dockerfile .
    docker run -t -i t-rex-tileserver/t-rex --version

    # Copy generated DEB package
    deb=$(docker run --entrypoint="" t-rex-tileserver/t-rex ls / | grep .deb)
    docker run --entrypoint="" -v /tmp:/var/data/out t-rex-tileserver/t-rex cp /$deb /var/data/out/
    popd
}

if [ $TRAVIS_OS_NAME = linux ]; then
    dockerimg
fi
# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi

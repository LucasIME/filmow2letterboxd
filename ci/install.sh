set -ex

main() {
    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-musl
        sort=sort
    else
        target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    case $TARGET in
        nightly-x86_64-unknown-linux-gnu)
            rustup component add rustc --toolchain nightly-x86_64-unknown-linux-gnu
            ;;
        nightly-x86_64-apple-darwin)
            rustup component add rustc --toolchain nightly-x86_64-apple-darwin
            ;;
    esac

    # This fetches latest stable release
    # local tag=$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross \
    #                    | cut -d/ -f3 \
    #                    | grep -E '^v[0.1.0-9.]+$' \
    #                    | $sort --version-sort \
    #                    | tail -n1)
    

    # latest version removed OpenSSL and broke our setup. Trying to pin to 0.1.16.
    local tag="v0.1.16"
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag $tag \
           --target $target
}

main
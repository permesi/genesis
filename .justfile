set shell := ["zsh", "-uc"]

branch := if `git rev-parse --abbrev-ref HEAD` == "main" { "latest" } else { `git rev-parse --abbrev-ref HEAD` }

build: test
  # echo <token> | docker login ghcr.io -u <user> --password-stdin
  # podman run --rm -it --env-file /root/genesis.env  ghcr.io/permesi/genesis:latest sh
  docker build -t genesis .
  docker tag genesis ghcr.io/permesi/genesis:{{ branch }}
  docker push ghcr.io/permesi/genesis:{{ branch }}

test: clippy
  cargo test

clippy:
  cargo clippy --all -- -W clippy::all -W clippy::nursery -D warnings

coverage:
  CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
  grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
  firefox target/coverage/html/index.html
  rm -rf *.profraw

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

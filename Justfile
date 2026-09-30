set shell := ["zsh", "-cu"]

default:
    just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check --workspace --all-targets

check-all:
    cargo check --workspace --all-targets --all-features

check-features:
    cargo check --workspace --all-targets --no-default-features
    cargo check --workspace --all-targets --all-features

test:
    cargo test --workspace --all-targets --all-features

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

cordial-quality:
    cordial quality

cordial-report:
    cordial view findings/quality-report.md

cordial-gate:
    cordial quality --deny-open

phase-check: fmt-check check-all check-features test clippy cordial-quality cordial-report

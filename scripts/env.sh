# Project paths
export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_INSTALL_ROOT="$PROJECT_ROOT/.tools"

# Tools versions for this project
export EXPECTED_RUST="1.92.0" # based on rust-toolchain.toml
export BINSTALL_VERSION="1.16.5"
export SBF_LINKER_VERSION="0.1.6"
export SOLANA_CLI_VERSION="3.1.5"

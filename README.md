# Korgs Program Library

A monorepo of Solana programs.

## Prerequisites

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install direnv (optional but recommended):

```bash
# macOS
brew install direnv

# Linux
sudo apt install direnv  # or your package manager
```

Add to your shell config (`~/.bashrc`, `~/.zshrc`, etc):

```bash
eval "$(direnv hook bash)"  # or zsh, fish, etc
```

## Setup

```bash
# Clone and enter the repo
cd korgs-program-library

# Allow direnv (if using)
direnv allow

# Install tools
setup-env

# Verify environment
check-env
```

## Scripts

- `setup-env` - Install all required tools
- `teardown-env` - Remove installed tools and clean build artifacts
- `check-env` - Verify environment is configured correctly

## Tool Versions

See `scripts/env.sh` for pinned tool versions.

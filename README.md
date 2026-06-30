# QuantumVM

An OpenQASM 3.0 language server and runtime written in Rust.

## Features

- **Statevector Quantum Simulation**: Multi-qubit simulation with parallelized gate operations via `rayon`
- **Full LSP Server**: `qasm-lsp` provides diagnostics, go-to-definition, references, hover, completions, signature help, rename, document symbols, and semantic tokens
- **Static Type Checking**: Type inference, error/warning/hint diagnostics, and cross-file reference tracking
- **Gate Modifiers**: `ctrl`, `negctrl`, `inv`, and `pow` gate composition
- **Pretty Error Reporting**: Color-coded diagnostics with source snippets via `ariadne`

## Installation

### From GitHub Releases

Download the latest binary for your platform from the [releases page](https://github.com/wiji1/QuantumVM/releases).

### From Source

```bash
git clone https://github.com/wiji1/QuantumVM
cd QuantumVM
cargo build --release
```

The build produces two binaries: `QuantumVM` (CLI runtime) and `qasm-lsp` (LSP server).

## Usage

```bash
quantumvm program.qasm
quantumvm program.qasm --input n=5 --input theta=3.14
quantumvm program.qasm --inputs params.json

```
See [docs/input-system.md](docs/input-system.md) for details on input declarations and value formats.


### IDE Integration

- **VS Code**: Install the [QASM Language Support](https://github.com/wiji1/QuantumVM-VSCode) extension.
- **JetBrains**: Install the [QASM Language Support](https://github.com/wiji1/QuantumVM-Jetbrains) plugin.

Both plugins launch `qasm-lsp` automatically and provide diagnostics, go-to-definition, references, hover, completions, signature help, rename, document symbols, and semantic tokens.

## Development

```bash
cargo test          # Run all tests
cargo clippy        # Lint
cargo build         # Build (debug)
```

The CI workflow (`.github/workflows/ci.yml`) runs tests and clippy on every push to `main` and on pull requests. Builds are tested on Linux, macOS, and Windows.

## Contributing

Contributions are welcome. Open an issue or pull request on [GitHub](https://github.com/wiji1/QuantumVM).

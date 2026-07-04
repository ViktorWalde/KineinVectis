# Comandos de build e verificacao

Sequencia padrao para copiar e colar quando quiser validar e atualizar os
binarios usados pelo launcher.

```bash
cargo fmt --all --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
scripts/verificar-cpp.sh
cmake --build --preset dev-local
cargo build --release -p kernwerk-core
cmake --build --preset dev-local-release
```

Execucao manual da IDE:

```bash
./scripts/kernwerk-studio
```

Quando o preset local ainda nao existir/configurar:

```bash
cmake --preset dev-local
cmake --preset dev-local-release
```

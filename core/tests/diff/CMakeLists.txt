# Differential test: the same request to the Rust core (the oracle) and to the C++ core.
#
# Documented in: docs/engenharia/estrategia-de-testes.md, section "Diferencial"
#
# The requests go over the JSON-RPC wire, and the responses are compared after normalization.
# Empty until the C++ core answers its first method.

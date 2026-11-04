# Fixture C de host

Projeto C11 próprio do repositório, sob AGPL-3.0-only: biblioteca estática com
duas unidades, header público, header gerado, executável e teste CTest.
O header recusa compilação como C++; `_Generic` exercita o dialeto C11.

Requer CMake ≥ 3.24, Ninja e Clang. Copiar para uma área de teste em disco;
abrir a cópia na IDE e usar **Compilar**, sem configure separado. O preset
`host` seleciona Clang e Debug; o core conserva `.kinein/build` como build dir.
O teste Rust `c_host` usa o despacho real, CMake/Clang/CTest reais e sugestões
LSP controladas para verificar configuração, recuperação e aplicação de edits.
Navegação com clangd real e depuração GDB/DAP precisam de provas próprias.

Reprodução independente para conferir o programa e o runner:

```bash
cmake --preset host -B .kinein/build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
cmake --build .kinein/build --parallel 2
ctest --test-dir .kinein/build --output-on-failure
```

O resultado esperado é `sample_values` aprovado e o executável `c_host` com
exit code zero. Alterar `SAMPLE_BIAS` no CMake deve regenerar flags e header;
o próximo build precisa atualizar o contexto da IDE. Alterar só o nome da
definição `sample_sum` em `src/sum.c` provoca erro de ligação; restaurar a
definição permite reconstruir e repetir o teste.

As mutações de conteúdo da CDB, sincronização do servidor inicial e preservação
de documentos usam o runner existente com `--suite c-host`. Ele exige baseline
aprovado, aplica cada mutação em cópia descartável e exige falha do teste
correspondente; erro de compilação não conta como mutante eliminado. Usar
target de QA aquecido, temporários em disco e o escopo de memória documentado.

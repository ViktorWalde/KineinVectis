# Checklist de validação

## Assets

```text
[ ] geometria válida em grade lógica 24×24.
[ ] fundo transparente e documento de alto contraste.
[ ] arte ocupa aproximadamente 82–90% do canvas lógico.
[ ] leitura verificada a 16 e 20 px.
[ ] nenhum filtro, blur ou sombra pesada.
```

## Árvore

```text
[ ] 16 px legível no modo compacto.
[ ] 20 px legível no modo padrão.
[ ] 24 px legível no modo confortável.
[ ] linha selecionada não apaga as cores.
[ ] alinhamento vertical consistente.
[ ] nenhum escalonamento fracionário.
[ ] nome do arquivo não encosta no ícone.
```

## Semântica

```text
[ ] CMakeLists.txt vence a regra de .txt.
[ ] .h recebe o ícone H.
[ ] .hpp recebe o ícone H++.
[ ] .c recebe C e .cpp recebe C++.
[ ] Cargo.toml e Cargo.lock recebem Cargo.
[ ] Makefile, GNUmakefile e *.mk recebem Make.
[ ] pyproject.toml recebe projeto Python.
[ ] package.xml e *.launch.xml recebem ROS 2 XML.
[ ] compose.yaml recebe Docker YAML.
[ ] config.yaml não recebe Docker YAML.
[ ] .sql recebe SQL.
[ ] arquivos de configuração recebem project-config.
[ ] nomes especializados futuros podem substituir o fallback.
```

## Temas

```text
[ ] fundo claro.
[ ] fundo escuro.
[ ] linha selecionada clara.
[ ] linha selecionada escura.
[ ] modo de alto contraste avaliado.
[ ] daltonismo não elimina toda a distinção.
```

## Desempenho

```text
[ ] ausência de reload de assets durante scroll.
[ ] Canvas repinta somente ao mudar nome, cor ou tamanho.
[ ] tamanho não é animado.
[ ] 5.000 linhas testadas.
[ ] abertura de pasta medida.
[ ] frame time monitorado.
```

# Checklist de validação

## Assets

```text
[ ] PNG RGBA válido em canvas 64×64.
[ ] fundo transparente.
[ ] arte ocupa aproximadamente 82–90% do canvas.
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
[ ] .cpp continua recebendo o ícone C/C++.
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
[ ] cache habilitado.
[ ] ausência de reload durante scroll.
[ ] sourceSize não é animado.
[ ] 5.000 linhas testadas.
[ ] abertura de pasta medida.
[ ] frame time monitorado.
```

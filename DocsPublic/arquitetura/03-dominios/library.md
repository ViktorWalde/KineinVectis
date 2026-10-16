# `library.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `library.*` — o catálogo curado de bibliotecas C/C++

Domínio novo no protocolo `0.68.0` (etapa 19 do `roadmaps/35`). **A IDE endossa
o que oferece**: cada entrada carrega licença verificada, versão pinada e a
frase que explica o que a biblioteca faz.

```text
library.list {}                  -> { libraries: [LibraryInfo] }
library.plan { id, target }      -> LibraryPlan
```

**Nenhum dos dois exige workspace aberto**, e o domínio é **stateless** — o
catálogo é estático e a disponibilidade se lê do filesystem. Nada depende do
estado do `Core`, e o handler não recebe `self`.

`status` é `detected` | `notDetected`. **`notDetected` não é "não existe":** a
detecção olha os diretórios de config package conhecidos, e o usuário pode ter
um `CMAKE_PREFIX_PATH` próprio. Por isso, quando nada foi achado, o
`LibraryPlan` devolve `searchedPaths` — *"não achei, e olhei aqui"* é acionável;
*"não achei"* manda adivinhar.

O `LibraryInfo` carrega `standardLineage` quando existe o sinal FORTE — a
biblioteca ter sido adotada no padrão ISO, ou passado pela revisão formal do
Boost. **Não há certificação oficial de biblioteca C++:** o WG21 padroniza a
linguagem e a biblioteca padrão, e a Standard C++ Foundation declara que seu
objetivo é reduzir barreiras para *adotar* bibliotecas no próprio Standard — não
certificar as de terceiros. O campo registra o sinal que existe de verdade em
vez de inventar um selo que ninguém emite; hoje só `fmt` o tem
(virou `std::format` no C++20).

**O plano não escreve nada.** Ele devolve passos que nomeiam a Configuration
Action que os executaria, e quem escreve continua sendo o domínio
`configaction`, com o preview e o consentimento que ele já tem. Dois escritores
do mesmo arquivo de build é a forma de eles divergirem em silêncio — e o
critério de aceite do corte é mecânico: `grep -c "CMakeLists"
crates/kinein-core/src/library/` tem que voltar **0**, inclusive em comentário.

**`find_package` ou `FetchContent` sai da MEDIÇÃO, não de um campo do
catálogo.** Se o config package está no sistema, usa-se ele; baixar e compilar o
que já está instalado é desperdício que o usuário paga em tempo de build. A
primeira versão tinha um enum com três variantes para isso e as sete entradas
auditadas saíram todas iguais — os lints reprovaram as variantes nunca
construídas, e estavam certos.

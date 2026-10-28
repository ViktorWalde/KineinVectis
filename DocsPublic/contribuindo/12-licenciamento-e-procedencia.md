<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# 12 — Licenciamento, distribuição e procedência

> **Classe: CONTRATO.** Política de licenciamento adotada em 2026-10-28.
> Decisão: [ADR-0011](../decisoes-adr/ADR-0011-copyleft-e-procedencia.md).
> Os textos integrais das licenças prevalecem sobre este guia explicativo.

## 1. Escopo das licenças

| Material | Licença e referência |
| --- | --- |
| Código próprio da IDE: Rust, C++, QML, scripts, testes e arquivos próprios de build e configuração | **AGPL-3.0-only**, [LICENSE](../../LICENSE) |
| Textos e diagramas próprios da documentação, inclusive arquivos Markdown da raiz | **CC-BY-SA-4.0**, [texto integral](../../LICENSE-CC-BY-SA-4.0.txt), ressalvadas as exceções abaixo |
| Trechos de código na documentação | Licença do código de origem; exemplos próprios de código seguem AGPL-3.0-only, salvo indicação específica |
| Ícones com avisos de licença próprios | Termos dos avisos de [ícones de arquivo](../iconografia/icones-de-arquivo/LICENSE-NOTICE.md) e [sistema visual](../iconografia/sistema-visual/02_QML_SVG_IMPLEMENTATION/LICENSE_ICONS.txt) |
| Metadados AppStream | CC0-1.0, conforme `metadata_license`; `project_license` identifica separadamente a licença da IDE |
| Dependências, ferramentas, citações, imagens e outros materiais de terceiros | Licenças e créditos dos respectivos titulares |
| Textos das licenças e do DCO | Permissões dos próprios documentos; reprodução sem alteração |
| Versões e materiais anteriormente disponibilizados sob MIT/Apache-2.0 | Permissões anteriores preservadas, conforme §2 |

Uma licença específica válida de arquivo ou componente prevalece sobre a
regra geral para aquele material. A inclusão no repositório ou no AppImage
não transfere à Kinein Vectis os direitos sobre componentes de terceiros.

O código próprio abrangido é disponibilizado nos termos da **GNU Affero
General Public License, versão 3 somente**, publicada pela Free Software
Foundation. Pode ser redistribuído e modificado nos termos dessa licença.
É fornecido sem garantia, inclusive sem garantia implícita de
comercialização ou adequação a uma finalidade específica, na extensão
permitida pela legislação. Consulte [LICENSE](../../LICENSE).

O [NOTICE](../../NOTICE) e [AUTHORS.md](../../AUTHORS.md) identificam a autoria.
A política de identidade está em [TRADEMARKS.md](../../TRADEMARKS.md).

## 2. Transição do licenciamento permissivo

O snapshot imediatamente anterior à migração é o commit
`bf9ec1726621ac2f32c02cbeb550581b896fea49`. Essa referência identifica a
fronteira técnica; não constitui prova isolada de autoria, publicação ou
titularidade. O histórico e os artefatos já existentes não são reescritos.

As permissões anteriormente concedidas validamente sob `MIT OR Apache-2.0`
continuam disponíveis para as respectivas cópias e partes, inclusive em
forks. A migração não permite retirar essas permissões ou exigir AGPL sobre
um desenvolvimento independente feito somente a partir delas.

Novas contribuições próprias abrangidas passam a entrar sob AGPL-3.0-only,
sem alternativa MIT/Apache automática. A simples presença de
[LICENSE-MIT.txt](../../LICENSE-MIT.txt) e
[LICENSE-APACHE-2.0.txt](../../LICENSE-APACHE-2.0.txt) não oferece essa
alternativa para código novo. Esses textos e os avisos aplicáveis são
mantidos para o material que continuam regendo.

A licença do conjunto não apaga os termos das partes incorporadas.
Relicenciar conteúdo exige direitos suficientes ou permissão compatível;
esta política não substitui autorização de terceiros.

## 3. Obrigações e limites do copyleft

| Operação | Consequência principal |
| --- | --- |
| Executar a IDE, inclusive comercialmente | Permitido; não exige pagamento ao projeto |
| Estudar ou modificar uma cópia privada | Não exige, por si só, publicação universal nem envio de patch ao projeto |
| Distribuir fontes abrangidos | Preservar os avisos, fornecer a licença e identificar modificações conforme §§4–5 da AGPL |
| Distribuir binários abrangidos | Fornecer o Código-Fonte Correspondente por modalidade admitida no §6; cumprir os demais requisitos aplicáveis |
| Oferecer versão modificada com interação remota pela rede | Oferecer aos usuários dessa interação acesso gratuito ao Código-Fonte Correspondente, conforme §13 |
| Criar um fork | Permitido, preservadas as obrigações de licença, atribuição e identificação de alterações |
| Escrever firmware ou outro programa independente usando a IDE | A licença da ferramenta não se estende automaticamente ao programa produzido |

Código-Fonte Correspondente inclui a forma preferida para modificar a obra
e os scripts necessários para gerar, instalar e executar o código objeto,
nas condições e com as exclusões definidas no §1. Um arquivo contendo apenas
o executável ou um link para fontes de outra versão não basta. Quando
aplicável, devem ser fornecidas as Informações de Instalação do §6.

Conexão de rede da IDE, uso de uma API externa ou comunicação local por
JSON-RPC não acionam automaticamente o §13. É preciso analisar a versão
modificada e a interação remota efetivamente oferecida. Da mesma forma,
separar programas em processos ou containers não comprova, sozinho, que
sejam obras independentes: acoplamento, protocolo e material incorporado
precisam ser avaliados.

Não se exige contribuir no repositório oficial nem publicar todo trabalho
privado na internet. A obrigação de disponibilização depende das hipóteses
e dos destinatários previstos na licença. Uma política de aceitação de
contribuições não amplia essas obrigações para todos os usuários.

## 4. Documentação livre

A documentação própria pode ser copiada, traduzida, adaptada e utilizada
comercialmente sob CC BY-SA 4.0. Ao compartilhá-la, devem ser atendidos os
requisitos de atribuição, referência à licença e indicação de alterações.
Adaptações compartilhadas devem usar uma licença admitida pela cláusula
ShareAlike. Não se podem impor restrições adicionais ou medidas tecnológicas
que impeçam os direitos concedidos.

Créditos e referências de terceiros permanecem identificados. A licença de
documentação não concede direitos sobre marcas, dados pessoais ou materiais
de terceiros além daqueles efetivamente licenciados.

## 5. Contribuições e DCO

Adota-se **inbound = outbound**: a contribuição entra sob a licença que
rege o material de destino. Não há cessão obrigatória de copyright nem
autorização geral para relicenciamento proprietário.

Para novas contribuições aceitas, o contribuidor deve ler o
[DCO 1.1](../../DCO.txt) e acrescentar seu `Signed-off-by` aos commits,
por exemplo com:

```sh
git commit -s
```

O sign-off certifica a procedência e o direito de contribuir nas condições
do DCO. É uma declaração pública, distinta de assinatura criptográfica.
Cada pessoa certifica seu próprio trabalho; não se adiciona sign-off de
terceiro sem sua autorização. O histórico anterior à política é preservado.

O DCO não substitui autorização de quem detenha direitos institucionais.
Quando necessária, aplica-se a [política institucional](13-colaboracao-institucional.md).
Colaboradores individuais sem esse impedimento seguem o fluxo normal,
sem acordo institucional adicional.

Arquivos novos de código próprio devem identificar a licença em comentário
compatível com a linguagem, após o shebang quando houver:

```rust
// SPDX-License-Identifier: AGPL-3.0-only
```

Textos próprios novos podem usar:

```html
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
```

Avisos de copyright devem indicar os titulares reais. Não substituir
créditos de terceiros nem marcar como próprio material apenas importado.
Nos arquivos existentes sem cabeçalho individual, vale o escopo do §1;
esta migração não declara uma auditoria REUSE completa do repositório.

## 6. Dependências e distribuição verificável

As licenças das dependências são verificadas pelo `cargo-deny`. A lista
permissiva de dependências continua em `deny.toml`; exceções por nome
autorizam AGPL-3.0-only apenas nos crates próprios. A migração não aprova
automaticamente a incorporação de novas dependências GPL/AGPL ou de qualquer
outra licença ainda não auditada.

Antes de distribuir uma versão:

1. Conferir licença, origem, versão e avisos dos componentes incluídos,
   inclusive bibliotecas Qt e dependências transitivas.
2. Manter os identificadores SPDX coerentes nos manifestos Cargo,
   AppStream e nos materiais distribuídos.
3. Incluir os textos de licença, atribuições e avisos aplicáveis; os textos
   oficiais em inglês são mantidos sem alteração.
4. Disponibilizar os fontes correspondentes ao binário exato e o material
   de build necessário, por uma modalidade válida da AGPL e das licenças
   de terceiros. Cobrir a oferta de rede quando aplicável.
5. Conferir as obrigações próprias das bibliotecas redistribuídas. A AGPL
   da IDE não substitui, por exemplo, os requisitos LGPL dos componentes Qt.

As regras de instalação e o smoke estrutural exigem os avisos da IDE.
`verificar-appimage.sh --invariantes` também confere a coerência das licenças
no workspace, nos crates, no AppStream e no aviso de distribuição. A cópia
de distribuição preserva os documentos legais explicitamente listados;
uma seleção parcial não certifica Código-Fonte Correspondente completo.
Gates técnicos
verificam arquivos e metadados; não certificam titularidade, não substituem
a auditoria dos componentes e não provam sozinhos cumprimento jurídico.

## 7. Serviços, personalizações e autonomia

Suporte, treinamento, implantação, manutenção e integrações sob encomenda
podem ser remunerados. O contrato deve distinguir esses serviços das
liberdades concedidas pelo software. O término do serviço não deve ser
usado nos contratos do projeto como penalidade pelo exercício dos direitos
de cópia, modificação ou redistribuição.

Uma personalização que incorpore código abrangido continua sujeita à AGPL
nas hipóteses aplicáveis. Dados, credenciais e programas independentes
podem permanecer privados. A denominação comercial “sob medida” não cria
exceção ao copyleft. Uma licença comercial alternativa exigiria direitos
suficientes sobre todo o material envolvido; o DCO não os concede.

Contribuição, patrocínio ou cessão de infraestrutura não concedem, por si
sós, administração do repositório, direito sobre a identidade oficial ou
poder de veto sobre novas versões. Os direitos de cada contribuidor são
preservados; atribuição não significa controle da governança.

A AGPL não garante publicação universal de alterações privadas, acesso
permanente a equipamentos ou continuidade de contratos comerciais. A
controvérsia documentada sobre contratos de suporte do RHEL não deve ser
tratada como uma lacuna definitivamente resolvida pela troca de licença.
Esta política adota reciprocidade padronizada e independência operacional,
sem prometer impedir toda disputa ou todo fork.

## 8. Referências

- [GNU AGPLv3 — texto oficial](https://www.gnu.org/licenses/agpl-3.0.html).
- [AGPLv3 aprovada pela OSI](https://opensource.org/license/agpl-3-0).
- [CC BY-SA 4.0 — texto jurídico](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en).
- [Developer Certificate of Origin 1.1](https://developercertificate.org/).
- [Identificador SPDX da AGPL](https://spdx.org/licenses/AGPL-3.0-only.html).
- [Regras SPDX do kernel Linux](https://docs.kernel.org/process/license-rules.html).
- [Compatibilidade Apache-2.0/GPL](https://apache.org/licenses/GPL-compatibility.html).
- [Análise da Software Freedom Conservancy sobre RHEL e GPL](https://sfconservancy.org/blog/2023/jun/23/rhel-gpl-analysis/).

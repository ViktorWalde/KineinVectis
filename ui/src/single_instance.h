#pragma once

#include <QObject>
#include <QString>

#include <functional>
#include <memory>
#include <optional>

// UMA JANELA POR PASTA (P0, decisao do autor em 2026-09-26).
//
// `kinein ~/projeto` com aquele projeto JA' ABERTO deve trazer a janela
// existente para a frente, e nao abrir uma segunda. Pasta diferente continua
// abrindo janela nova — sao processos independentes, e e' assim que ficam.
//
// POR QUE ISTO IMPORTA, E NAO E' CONFORTO. Nao ha' lock de workspace: duas
// janelas na mesma pasta sao duas donas do `.kinein/`, escrevendo o mesmo
// `session.json` e o mesmo indice. A ultima a fechar apaga o que a outra
// gravou, sem erro e sem aviso. Medido em 2026-09-26.
//
// COMO FUNCIONA. Cada janela que abre um workspace escuta num socket de
// dominio Unix dentro do `XDG_RUNTIME_DIR`, cujo nome vem de um HASH do
// caminho canonico. Quem chega depois tenta conectar: se alguem atende e
// CONFIRMA o caminho, o recem-chegado pede foco e sai; se ninguem atende, o
// socket orfao e' removido e ele assume.
//
// O HASH NAO DECIDE SOZINHO. Ele so' escolhe o nome do arquivo — o caminho
// inteiro viaja na mensagem e e' conferido do outro lado. Colisao de hash
// entao nao pode fazer uma pasta se passar por outra; no maximo faz duas
// pastas disputarem um nome de arquivo, e a segunda descobre que a casa nao e'
// dela e segue como janela nova.
//
// LIMITE DITO, E MEDIDO. Trazer para a frente a janela de OUTRO processo e'
// privilegio do compositor. No Wayland do GNOME — que e' o ambiente do autor,
// e a IDE e' cliente Wayland nativo — um pedido sem token de ativacao vira
// "janela pronta" na barra, e nao foco. O que este arquivo garante em qualquer
// ambiente e' o que importa: NAO abrir a segunda janela, e dizer no terminal o
// que aconteceu. O foco e' o melhor esforco por cima disso.
//
// UM FIO POR SISTEMA (DocsPublic/roadmaps/60 §3.2, W3). O protocolo e as
// decisoes (nome, mensagem, conferencia do caminho) sao estes, iguais nos dois
// sistemas, e moram no `single_instance.cpp`. O transporte muda: socket de
// dominio Unix no `single_instance_unix.cpp`; no Windows, named pipe pelo
// `QLocalServer`, so' do proprio usuario (`single_instance_windows.cpp`, decisao
// D4 do autor). Ali o "descritor" das funcoes abaixo e' so' um numero que
// identifica o servidor.
namespace kinein {

/// Onde os sockets desta sessao moram, ou vazio quando nao ha' onde.
///
/// No Unix e' o `XDG_RUNTIME_DIR`; vazio e' um caso real (container magro,
/// sessao sem systemd) e nao um erro: a IDE segue sem a coordenacao, abrindo
/// janela como sempre abriu. No Windows e' um prefixo com o nome do usuario,
/// porque o pipe e' da maquina inteira.
[[nodiscard]] QString runtimeDirectory();

/// Chama `onReady` quando alguem conecta no descritor de `listenFor`.
///
/// O objeto devolvido e' o dono da escuta: destrui-lo a encerra. Ele deve
/// morrer ANTES do descritor (`releaseSocket`). `nullptr` quando nao ha' o que
/// observar.
[[nodiscard]] std::unique_ptr<QObject> watchIncoming(int listenFd,
                                                     const std::function<void()>& onReady);

/// O caminho do socket desta pasta, ou vazio quando nao da' para ter um.
///
/// Vazio acontece de verdade: sem `XDG_RUNTIME_DIR` (sessao sem systemd, um
/// container magro) nao ha' onde por o socket, e o programa segue sem a
/// coordenacao em vez de inventar um diretorio.
[[nodiscard]] QString socketPathFor(const QString& runtimeDir, const QString& workspacePath);

/// A linha que o recem-chegado manda. Termina em `\n`.
///
/// O token de ativacao viaja junto QUANDO EXISTE: e' o unico jeito de o
/// Wayland deixar a outra janela subir, e ele so' e' valido por um gesto.
[[nodiscard]] QString encodeRequest(const QString& workspacePath, const QString& activationToken);

/// O que o dono do socket leu.
struct Request
{
    /// A pasta que o recem-chegado quer abrir.
    QString workspacePath;
    /// Token de ativacao do XDG, ou vazio.
    QString activationToken;
};

/// Le' a linha; `nullopt` quando ela nao e' deste protocolo.
///
/// Recusar o que nao reconhece e' o ponto: um socket no `XDG_RUNTIME_DIR` e'
/// alcancavel por qualquer processo do mesmo usuario, e responder a qualquer
/// coisa que chegue seria obedecer a quem nao se identificou.
[[nodiscard]] std::optional<Request> parseRequest(const QString& line);

/// Duas descricoes da MESMA pasta?
///
/// Comparacao por caminho canonico ja' foi feita por quem chama; aqui sobra
/// normalizar a barra final, que `QDir::cleanPath` deixa passar na raiz.
[[nodiscard]] bool sameWorkspace(const QString& first, const QString& second);

/// Entrega o pedido a quem ja' tem esta pasta aberta.
///
/// `true` significa: alguem atendeu E confirmou que a pasta e' dele. O
/// chamador deve SAIR sem abrir janela. Qualquer outra coisa — socket
/// inexistente, orfao, sem resposta a tempo, resposta negativa — e' `false`,
/// e a vida segue como janela nova. Errar para o lado de ABRIR e' deliberado:
/// uma janela a mais e' um incomodo; uma janela a menos, com o autor achando
/// que a IDE ignorou o comando, e' um defeito.
[[nodiscard]] bool handOff(const QString& socketPath, const QString& workspacePath,
                           const QString& activationToken = QString{}, int timeoutMs = 400);

/// Assume o socket desta pasta. Devolve o descritor de escuta, ou -1.
///
/// Socket ORFAO — arquivo existe, ninguem escuta — e' removido e o lugar e'
/// assumido. Essa e' a situacao depois de um crash, e deixar a coordenacao
/// quebrada ate' o proximo reboot seria pior que o risco de remover.
[[nodiscard]] int listenFor(const QString& socketPath);

/// Aceita UM pedido do descritor e responde.
///
/// Devolve o pedido quando ele foi aceito e a pasta CONFERIU; `nullopt` quando
/// nao ha' ninguem, quando a linha nao e' deste protocolo, ou quando a pasta
/// e' outra (caso em que o outro lado recebe o "nao" e segue sozinho).
[[nodiscard]] std::optional<Request> acceptOne(int listenFd, const QString& myWorkspacePath);

/// Fecha o descritor e apaga o arquivo do socket.
void releaseSocket(int listenFd, const QString& socketPath);

/// A resposta do dono: sim, esta pasta e' minha.
[[nodiscard]] QString mineAnswer();
/// A resposta do dono: nao e', pode seguir.
[[nodiscard]] QString notMineAnswer();
/// Interpreta a resposta. Qualquer coisa que nao seja o "sim" exato e' um nao.
[[nodiscard]] bool answerIsMine(const QString& answer);

} // namespace kinein

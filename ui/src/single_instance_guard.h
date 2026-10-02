#pragma once

#include "single_instance.h"

#include <QObject>
#include <QSocketNotifier>
#include <QString>
#include <QtQml/qqmlregistration.h>

#include <memory>

// O LADO VIVO da instancia unica: esta janela avisa que esta pasta e' dela.
//
// Enquanto um workspace esta' aberto, este objeto escuta no socket daquela
// pasta. Quem chegar depois com `kinein <a mesma pasta>` recebe um "meu",
// desiste de abrir a segunda janela e — quando o compositor deixa — pede que
// esta suba.
//
// SEGUE O WORKSPACE. Trocar de pasta na mesma janela troca o socket: a antiga
// deixa de ser reivindicada (outra janela pode assumi-la) e a nova passa a
// ser. Fechar o workspace solta o socket.
//
// O QUE ELE NAO FAZ: decidir. Todas as decisoes — nome do socket, forma da
// mensagem, conferencia do caminho — vivem no `single_instance`, sem Qt GUI e
// com teste proprio. Aqui ha' descritor, notificador e ciclo de vida.
namespace kinein {

class SingleInstanceGuard : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    /// A pasta que esta janela tem aberta. Vazio solta o socket.
    Q_PROPERTY(
        QString workspacePath READ workspacePath WRITE setWorkspacePath NOTIFY workspacePathChanged)

public:
    explicit SingleInstanceGuard(QObject* parent = nullptr);
    ~SingleInstanceGuard() override;

    SingleInstanceGuard(const SingleInstanceGuard&) = delete;
    SingleInstanceGuard& operator=(const SingleInstanceGuard&) = delete;
    SingleInstanceGuard(SingleInstanceGuard&&) = delete;
    SingleInstanceGuard& operator=(SingleInstanceGuard&&) = delete;

    [[nodiscard]] QString workspacePath() const;
    void setWorkspacePath(const QString& path);

signals:
    void workspacePathChanged();

    /// Alguem pediu esta janela. `token` e' o do XDG, quando o terminal deu um.
    ///
    /// Quem sobe a janela e' o QML, que e' quem tem a `Window`. O token vai
    /// junto porque no Wayland ele e' a unica autorizacao que o compositor
    /// aceita — e sem ele o pedido vira "janela pronta" na barra, e nao foco.
    void activationRequested(const QString& token);

private:
    void release();
    void claim();
    void serveIncoming();

    QString m_workspacePath;
    QString m_socketPath;
    int m_listenFd = -1;
    // DONO EXPLICITO, e nao filho do Qt: o notificador nasce e morre com o
    // descritor que ele observa, e amarrar isso ao ciclo de vida do QObject
    // pai deixaria um notificador apontando para um fd ja' fechado.
    std::unique_ptr<QSocketNotifier> m_notifier;
};

} // namespace kinein

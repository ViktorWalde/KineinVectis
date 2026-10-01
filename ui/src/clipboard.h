// Ponte mínima para a área de transferência do sistema (fatia D2.2, DocsPublic/roadmaps/24).
//
// Utilitário de UI exposto ao QML como singleton `Clipboard`, para o terminal
// (e futuros consumidores) copiar/colar sem falar com o CoreClient — copiar/
// colar é UI, não IPC.

#pragma once

#include <QObject>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class Clipboard : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool filesAvailable READ hasFiles NOTIFY filesChanged)

public:
    explicit Clipboard(QObject* parent = nullptr);

    Q_INVOKABLE void setText(const QString& text);
    Q_INVOKABLE QString text() const;
    Q_INVOKABLE void setFiles(const QStringList& paths, bool cut);
    Q_INVOKABLE QStringList filePaths() const;
    Q_INVOKABLE QStringList localFilePathsFromUrls(const QVariantList& urls) const;
    Q_INVOKABLE QString localDirectoryPathFromUrls(const QVariantList& urls) const;
    Q_INVOKABLE QUrl localFileUrl(const QString& path) const;
    Q_INVOKABLE bool filesCut() const;
    Q_INVOKABLE void clearCutFileIfMatches(const QString& path);
    Q_INVOKABLE void removeCutFilesIfMatches(const QStringList& expectedPaths,
                                             const QStringList& movedPaths);
    [[nodiscard]] bool hasFiles() const;

signals:
    void filesChanged();
};

} // namespace kinein

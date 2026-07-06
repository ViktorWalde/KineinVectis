import QtQuick

Item {
    id: root

    property alias messagesModel: messageItemsModel

    visible: false

    ListModel {
        id: messageItemsModel
    }

    function initialize() {
        if (messageItemsModel.count > 0) {
            return;
        }
        messageItemsModel.append({
            role: "system",
            body: qsTr("Assistente KW ainda esta offline. Os providers de IA "
                       + "(Ollama local, GPT CLI, Claude CLI) chegam na Fase 7 do "
                       + "roadmap. Politica: nenhum codigo sai da maquina sem a sua "
                       + "confirmacao explicita.")
        });
    }

    function sendMessage(body) {
        const text = body.trim();
        if (text === "") {
            return;
        }
        messageItemsModel.append({ role: "user", body: text });
        messageItemsModel.append({
            role: "system",
            body: qsTr("Nenhum provider de IA esta configurado ainda. Os providers "
                       + "(Ollama local, GPT CLI, Claude CLI) chegam na Fase 7 do "
                       + "roadmap, sempre com confirmacao explicita antes de "
                       + "qualquer codigo sair da maquina.")
        });
    }
}

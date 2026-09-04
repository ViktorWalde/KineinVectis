pragma ComponentBehavior: Bound
import QtQuick

// O passo a passo OFICIAL para instalar o que falta, na distro detectada.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). Ideia do autor: a IDE mostrar,
// para quem esta comecando, o comando OFICIAL de instalacao da ferramenta —
// com o link da fonte ao lado, para conferir em vez de confiar.
//
// Ele resolve tambem o buraco que o proprio autor apontou antes: quando uma
// biblioteca nao esta' instalada, a IDE escreve `FetchContent` com uma URL do
// GitHub no CMakeLists dele. O caminho que DISPENSA a URL e' instalar o pacote
// no sistema, e ate' aqui a IDE dizia "instale" sem dizer COMO.
//
// A IDE NAO RODA NADA SOZINHA. "Enviar ao terminal" ESCREVE o comando no
// terminal da propria IDE, visivel, onde o autor le' a linha e responde o
// prompt de senha. Instalador silencioso com root dentro de um editor de texto
// e' poder que ninguem pediu.
Item {
    id: root

    property bool panelVisible: false
    property string distroName: ""
    property string family: ""
    property var tools: []
    property string errorText: ""

    // Ferramenta cujo passo a passo esta aberto. Uma so' por vez: a lista
    // inteira aberta viraria uma parede de comando.
    property string expandedId: ""

    signal listRequested()
    signal commandRequested(string command)

    visible: false

    function open() {
        panelVisible = true;
        listRequested();
    }

    function close() {
        panelVisible = false;
    }

    function toggle(id) {
        expandedId = expandedId === id ? "" : id;
    }

    function handleList(distro, newFamily, newTools) {
        distroName = distro;
        family = newFamily;
        tools = newTools;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method === "setup.list") {
            errorText = message;
        }
    }
}

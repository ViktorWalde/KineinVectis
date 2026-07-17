pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Estado de configuracoes (fatia M4.1). Consome settings.get/set do core:
// guarda o efetivo (default <- global <- workspace) e o que esta setado em
// cada escopo, aplica editorFontSize no Theme e expoe os flags para os
// consumidores (auto-close, format-on-save). Nao fala com o CoreClient
// direto — pede por sinal e recebe do roteador (guardrail docs/arquitetura/17).
Item {
    id: root

    // Efetivo (o que a UI aplica).
    property bool formatOnSave: false
    property int editorFontSize: 14
    property bool autoClosePairs: true
    property int explorerWidth: 280
    property int contextWidth: 360
    property int bottomPanelHeight: 260
    property int outlineWidth: 220
    property bool outlineCollapsed: false
    // Perfil de rigor do build/quality do usuario (M4.5): strict|balanced|relaxed.
    property string rigorProfile: "strict"
    // O que esta explicitamente setado no global (para a UI mostrar).
    property var globalValues: ({})
    property var workspaceValues: ({})
    property bool loaded: false
    property bool dialogVisible: false

    signal getRequested()
    signal setRequested(string scope, var values)
    signal resolved()

    visible: false

    Component.onCompleted: getRequested()

    function openDialog() {
        dialogVisible = true;
    }

    function closeDialog() {
        dialogVisible = false;
    }

    function handleResolved(effective, global, workspace) {
        formatOnSave = effective.formatOnSave === true;
        editorFontSize = effective.editorFontSize !== undefined
                ? effective.editorFontSize : 14;
        autoClosePairs = effective.autoClosePairs !== false;
        rigorProfile = effective.rigorProfile !== undefined
                ? effective.rigorProfile : "strict";
        explorerWidth = effective.explorerWidth !== undefined
                ? effective.explorerWidth : 280;
        contextWidth = effective.contextWidth !== undefined
                ? effective.contextWidth : 360;
        bottomPanelHeight = effective.bottomPanelHeight !== undefined
                ? effective.bottomPanelHeight : 260;
        outlineWidth = effective.outlineWidth !== undefined
                ? effective.outlineWidth : 220;
        outlineCollapsed = effective.outlineCollapsed === true;
        globalValues = global;
        workspaceValues = workspace;
        loaded = true;
        // Consumidor direto: o tamanho de fonte do editor.
        Theme.fontSizeEditor = editorFontSize;
        resolved();
    }

    // A UI edita o escopo GLOBAL no v1 (values e um objeto parcial).
    function setGlobal(values) {
        setRequested("global", values);
    }

    function hasPersistedLayout() {
        return globalValues.explorerWidth !== undefined
                || globalValues.contextWidth !== undefined
                || globalValues.bottomPanelHeight !== undefined
                || globalValues.outlineWidth !== undefined
                || globalValues.outlineCollapsed !== undefined
                || workspaceValues.explorerWidth !== undefined
                || workspaceValues.contextWidth !== undefined
                || workspaceValues.bottomPanelHeight !== undefined
                || workspaceValues.outlineWidth !== undefined
                || workspaceValues.outlineCollapsed !== undefined;
    }
}

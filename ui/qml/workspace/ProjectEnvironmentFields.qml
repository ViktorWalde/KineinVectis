pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root
    required property var controller
    readonly property var options: controller.projectOptions
    width: parent.width
    spacing: Theme.spacingSmall
    visible: controller.createLanguage !== "" && controller.createLanguage !== "empty"

    Row {
        width: parent.width
        spacing: Theme.spacingSmall
        Text {
            width: parent.width - refresh.width - parent.spacing
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("AMBIENTE INSTALADO")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
        }
        KvButton {
            id: refresh
            compact: true
            text: root.options.loading ? qsTr("Buscando…") : qsTr("Atualizar")
            enabled: !root.options.loading
            onClicked: root.options.refresh()
        }
    }
    ListView {
        width: parent.width
        height: Math.min(2, count) * 30
        clip: true
        model: root.options.forLanguage(root.controller.createLanguage)
        spacing: 2
        delegate: KvButton {
            required property var modelData
            width: ListView.view.width
            height: 28
            compact: true
            text: modelData.version.length > 84 ? modelData.version.slice(0, 81) + "…" : modelData.version
            tooltip: modelData.executable
            selected: root.options.executable === modelData.executable
            onClicked: root.options.executable = modelData.executable
        }
    }
    KvTextField {
        width: parent.width
        height: 40
        label: qsTr("Executável (caminho alternativo permitido)")
        placeholder: qsTr("Automático — ferramenta do ambiente")
        text: root.options.executable
        onEdited: (value) => { root.options.executable = value; }
    }
    KvTextField {
        width: parent.width
        height: 40
        visible: root.controller.createLanguage !== "python"
        label: root.controller.createLanguage === "rust" ? qsTr("Edição Rust") : qsTr("Padrão da linguagem")
        placeholder: root.controller.createLanguage === "rust" ? qsTr("Padrão do Cargo; ou informe 2021, 2024…")
                     : root.controller.createLanguage === "c" ? qsTr("Padrão do compilador; ou informe 90, 99, 11, 17, 23…")
                     : qsTr("Padrão do compilador; ou informe 98, 11, 17, 20, 23…")
        text: root.options.standard
        onEdited: (value) => { root.options.standard = value; }
    }
    Text {
        width: parent.width
        text: root.options.errorText !== "" ? root.options.errorText
              : qsTr("Escolha uma instalação ou informe seu caminho. Novas versões são instaladas em Ferramentas.")
        color: root.options.errorText !== "" ? Theme.errorSoft : Theme.textMuted
        wrapMode: Text.WordWrap
        font.pixelSize: Theme.fontSizeCaption
    }
}

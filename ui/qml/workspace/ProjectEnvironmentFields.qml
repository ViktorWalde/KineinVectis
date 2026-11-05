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
    KvSelect {
        width: parent.width
        choices: root.options.forLanguage(root.controller.createLanguage).map(entry => ({
            value: entry.executable, label: entry.version + " — " + entry.executable
        }))
        value: root.options.executable
        busy: root.options.loading
        emptyLabel: qsTr("Automático — ferramenta do ambiente")
        accessibleLabel: qsTr("Instalação local")
        customPlaceholder: qsTr("Caminho do executável instalado")
        onEdited: value => { root.options.executable = value; }
    }
    Text {
        width: parent.width
        visible: root.options.executable !== ""
        text: root.options.executable
        textFormat: Text.PlainText
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeCaption
        wrapMode: Text.WrapAnywhere
    }
    Text {
        text: root.controller.createLanguage === "rust" ? qsTr("EDIÇÃO RUST") : qsTr("PADRÃO / DIALETO (OPCIONAL)")
        visible: root.controller.createLanguage !== "python"
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeMicro
    }
    KvSelect {
        width: parent.width
        visible: root.controller.createLanguage !== "python"
        choices: root.options.standardsFor(root.controller.createLanguage)
        emptyLabel: root.controller.createLanguage === "rust" ? qsTr("Padrão do Cargo") : qsTr("Da ferramenta — sem padrão imposto")
        accessibleLabel: root.controller.createLanguage === "rust" ? qsTr("Edição Rust") : qsTr("Padrão da linguagem")
        customPlaceholder: root.controller.createLanguage === "c" ? qsTr("Ex.: 17, gnu17, c2y, iso9899:1990…") : qsTr("Informe outro valor")
        value: root.options.standard
        onEdited: value => { root.options.standard = value; }
    }
    Text {
        width: parent.width
        text: root.options.errorText !== "" ? root.options.errorText
              : root.options.loading ? qsTr("Consultando instalações locais…")
              : root.options.forLanguage(root.controller.createLanguage).length === 0
                ? qsTr("Nenhuma instalação detectada. Use Personalizado… para indicar um caminho; novas versões ficam em Ferramentas.")
                : qsTr("Selecione uma instalação detectada ou use Personalizado… para outro caminho. Novas versões ficam em Ferramentas.")
        color: root.options.errorText !== "" ? Theme.errorSoft : Theme.textMuted
        wrapMode: Text.WordWrap
        font.pixelSize: Theme.fontSizeCaption
    }
}

pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// QUAL ADAPTADOR roda o perfil (9a.3, arquitetura/39 §6.2.2). Burro: as
// opcoes vem do descritor do core pelo DataSourceKinds, e o gesto devolve o
// tipo escolhido. So' aparece quando o motor oferece mais de um.
Column {
    id: root

    property var provider: null
    property string current: "builtin"
    readonly property var choices: DataSourceKinds.installationOptions(root.provider)
    signal selected(string kind)

    objectName: "dataSourceAdapterField"
    visible: root.choices.length > 1
    spacing: Theme.spacingSmall

    Text {
        width: parent.width
        text: qsTr("Adaptador")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    KvSegmentedControl {
        width: parent.width
        current: root.current
        options: root.choices
        onSelected: value => root.selected(value)
    }

    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        text: qsTr("Interno roda dentro do core. O da IDE é o kinein-adapter-%1, instalado ao lado do core, num processo à parte.")
              .arg(root.provider ? root.provider.engine : "")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
}

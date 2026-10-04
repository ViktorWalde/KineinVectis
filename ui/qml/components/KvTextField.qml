import QtQuick
import KineinVectis

// O CAMPO DE TEXTO da IDE: caixa, entrada, placeholder, foco.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-10-03). Havia 32 campos montados a mao —
// um `Rectangle` com um `TextInput` dentro e um `Text` de placeholder por
// cima — e cada um com a sua altura, a sua borda e o seu jeito de mostrar
// foco (uns com placeholder sumindo no foco, outros nao; uns com selecao
// ambar, outros azul do sistema). Pedido do autor no fechamento da 0.3.9:
// modernizar todo controle no estilo antigo, com a mesma linguagem dos
// interruptores (KvToggleChip). Um dono para a forma, 32 usos.
//
//   [🔍 filtrar…                       ×]   pairar: borda acende
//   ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓   foco: borda ambar + anel suave
//
// O FOCO E' DO CAMPO INTEIRO. A raiz e' um FocusScope: `campo.forceActiveFocus()`
// chega na entrada, e `activeFocus` do campo diz se ela tem o foco. As teclas
// que a entrada nao usa (Esc, setas, Enter depois do `accepted`) sobem para o
// campo, entao quem usa escreve `Keys.onEscapePressed` no proprio KvTextField.
// Quem precisa da tecla ANTES da entrada (Enter com Shift, por exemplo) usa
// `keyPressed`, que roda antes e pode aceitar o evento.
FocusScope {
    id: root

    property alias text: input.text
    property string placeholder: ""
    // Mono para codigo, caminho, nome de alvo; texto de interface sem.
    property bool codeFont: true
    property int pixelSize: Theme.fontSizeBody
    property alias echoMode: input.echoMode
    property alias readOnly: input.readOnly
    property alias validator: input.validator
    property alias inputMethodHints: input.inputMethodHints
    property alias maximumLength: input.maximumLength
    property alias horizontalAlignment: input.horizontalAlignment
    property alias cursorPosition: input.cursorPosition
    property alias selectedText: input.selectedText
    // Icone a esquerda (busca, filtro) e o × que limpa.
    property string iconName: ""
    property bool clearable: false
    // Valor recusado: a borda em vermelho brando (o motivo vai em outro lugar).
    property bool error: false
    property color fill: root.readOnly ? Theme.background2 : Theme.background0
    // A entrada, para o raro uso que precisa dela (posicao do cursor, etc.).
    readonly property alias input: input

    signal edited(string text)
    signal accepted()
    signal editingFinished()
    signal keyPressed(var event)

    readonly property bool hovered: hover.hovered

    function selectAll() {
        input.selectAll();
    }

    function clear() {
        input.clear();
        root.edited("");
    }

    implicitWidth: 200
    implicitHeight: 28
    Accessible.role: Accessible.EditableText
    Accessible.name: root.placeholder

    // O anel de foco: suave, por fora da borda.
    Rectangle {
        anchors.fill: parent
        anchors.margins: -2
        radius: box.radius + 2
        color: "transparent"
        border.width: 2
        border.color: root.error ? Theme.errorSoft : Theme.accent
        opacity: input.activeFocus ? 0.28 : 0

        Behavior on opacity {
            NumberAnimation { duration: Theme.motionFast }
        }
    }

    Rectangle {
        id: box

        anchors.fill: parent
        radius: Theme.radius
        color: root.fill
        border.width: 1
        border.color: root.error ? Theme.errorSoft
                      : input.activeFocus ? Theme.accent
                      : (root.hovered && !root.readOnly ? Theme.borderStrong : Theme.borderSoft)

        Behavior on border.color {
            ColorAnimation { duration: Theme.motionFast }
        }
    }

    HoverHandler {
        id: hover

        cursorShape: Qt.IBeamCursor
    }

    KvIcon {
        id: leading

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.iconName !== ""
        name: root.iconName === "" ? "search" : root.iconName
        size: 14
        active: input.activeFocus
    }

    TextInput {
        id: input

        anchors.left: leading.visible ? leading.right : parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: clearButton.visible ? clearButton.left : parent.right
        anchors.rightMargin: clearButton.visible ? 2 : Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        height: parent.height
        focus: true
        verticalAlignment: TextInput.AlignVCenter
        clip: true
        selectByMouse: true
        color: root.readOnly ? Theme.textSecondary : Theme.textPrimary
        selectionColor: Theme.accentDim
        selectedTextColor: Theme.textPrimary
        font.family: root.codeFont ? Theme.monoFont : Theme.uiFont
        font.pixelSize: root.pixelSize
        Keys.priority: Keys.BeforeItem
        Keys.onPressed: (event) => root.keyPressed(event)
        onTextEdited: root.edited(text)
        onAccepted: root.accepted()
        onEditingFinished: root.editingFinished()

        Text {
            anchors.fill: parent
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: input.horizontalAlignment
            visible: input.text === "" && input.preeditText === ""
            text: root.placeholder
            color: Theme.textMuted
            elide: Text.ElideRight
            font.family: input.font.family
            font.pixelSize: input.font.pixelSize
        }
    }

    KvIconButton {
        id: clearButton

        anchors.right: parent.right
        anchors.rightMargin: 2
        anchors.verticalCenter: parent.verticalCenter
        visible: root.clearable && input.text !== "" && !root.readOnly
        compact: true
        width: Math.min(22, root.height - 4)
        height: width
        iconSize: 12
        iconName: "close"
        tooltip: qsTr("Limpar")
        focusOnClick: false
        onClicked: {
            root.clear();
            input.forceActiveFocus();
        }
    }
}

pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CAMPO DE TEXTO da IDE: caixa, entrada, placeholder, foco.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-10-03). Havia 32 campos montados a mao —
// um `Rectangle` com um `TextInput` dentro e um `Text` de placeholder por
// cima — e cada um com a sua altura, a sua borda e o seu jeito de mostrar
// foco. Um dono para a forma, 32 usos.
//
// O DESENHO (refeito em 2026-10-04, relato do autor: "esses campos de
// digitacao estao ruins, precisam ser mais dinamicos"; a primeira versao
// tinha um anel ambar por fora da borda ambar, que lia como uma caixa grossa
// e parada):
//
//   repouso   [ 🔍 filtrar…                      ]  borda discreta
//   pairar    [ 🔍 filtrar…                      ]  o fundo clareia
//   foco      [ 🔍 alp|                        × ]  o icone acende, o cursor
//             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ambar pisca suave, e a
//                                                   linha ambar CRESCE do
//                                                   centro para as pontas
//   recusado  a linha e a borda em vermelho, e o campo treme uma vez
//
// O placeholder esmaece e desliza ao comecar a digitar; o × aparece com
// fade. Com `label`, o rotulo mora DENTRO do campo e sobe quando ele ganha
// foco ou texto (o rotulo flutuante) — a pergunta fica a vista mesmo depois
// de respondida.
//
// O FOCO E' DO CAMPO INTEIRO. A raiz e' um FocusScope: `campo.forceActiveFocus()`
// chega na entrada, e `activeFocus` do campo diz se ela tem o foco. As teclas
// que a entrada nao usa (Esc, setas, Enter depois do `accepted`) sobem para o
// campo, entao quem usa escreve `Keys.onEscapePressed` no proprio KvTextField.
// Quem precisa da tecla ANTES da entrada (Enter com Shift, por exemplo) usa
// `keyPressed`, que roda antes e pode aceitar o evento.
FocusScope {
    id: root

    property alias text: textInput.text
    property string placeholder: ""
    // O rotulo flutuante (opcional): dentro do campo, sobe no foco/texto.
    property string label: ""
    // Mono para codigo, caminho, nome de alvo; texto de interface sem.
    property bool codeFont: true
    property int pixelSize: Theme.fontSizeBody
    property alias echoMode: textInput.echoMode
    property alias readOnly: textInput.readOnly
    property alias validator: textInput.validator
    property alias inputMethodHints: textInput.inputMethodHints
    property alias maximumLength: textInput.maximumLength
    property alias horizontalAlignment: textInput.horizontalAlignment
    property alias cursorPosition: textInput.cursorPosition
    property alias selectedText: textInput.selectedText
    // Icone a esquerda (busca, filtro) e o × que limpa.
    property string iconName: ""
    property bool clearable: false
    // Valor recusado: vermelho e um tremor (o motivo vai em outro lugar).
    property bool error: false
    property color fill: root.readOnly ? Theme.background2 : Theme.background0
    // A entrada, para o raro uso que precisa dela (posicao do cursor, etc.).
    readonly property alias input: textInput

    signal edited(string text)
    signal accepted()
    signal editingFinished()
    signal keyPressed(var event)

    readonly property bool hovered: hover.hovered
    readonly property bool focused: textInput.activeFocus
    readonly property bool hasLabel: root.label !== ""
    // O rotulo sobe com foco ou texto.
    readonly property bool floated: root.focused || textInput.text !== "" || textInput.preeditText !== ""

    function selectAll() {
        textInput.selectAll();
    }

    function clear() {
        textInput.clear();
        root.edited("");
    }

    // O gesto do menu de contexto (TextMenuController). Colar e recortar
    // passam pelo `textEdited` da entrada, como se fossem digitados.
    function runMenuAction(action, selectionStart, selectionEnd) {
        textInput.forceActiveFocus();
        if (selectionEnd > selectionStart) textInput.select(selectionStart, selectionEnd);
        switch (action) {
        case "cut": textInput.cut(); break;
        case "copy": textInput.copy(); break;
        case "paste": textInput.paste(); break;
        case "selectAll": textInput.selectAll(); break;
        case "clear": root.clear(); break;
        }
    }

    implicitWidth: 200
    implicitHeight: root.hasLabel ? 44 : 30
    Accessible.role: Accessible.EditableText
    Accessible.name: root.hasLabel ? root.label : root.placeholder

    onErrorChanged: if (root.error) shake.restart()

    transform: Translate {
        id: shakeOffset
    }

    SequentialAnimation {
        id: shake

        NumberAnimation { target: shakeOffset; property: "x"; to: -4; duration: Theme.motionFast / 2 }
        NumberAnimation { target: shakeOffset; property: "x"; to: 3; duration: Theme.motionFast / 2 }
        NumberAnimation { target: shakeOffset; property: "x"; to: 0; duration: Theme.motionFast / 2 }
    }

    Rectangle {
        id: box

        anchors.fill: parent
        radius: Theme.radius
        // Clareia ao pairar e um pouco mais no foco: o campo "acorda".
        color: root.readOnly ? root.fill
               : root.focused ? Qt.lighter(root.fill, 1.35)
               : (root.hovered ? Qt.lighter(root.fill, 1.2) : root.fill)
        border.width: 1
        border.color: root.error ? Theme.errorSoft
                      : root.focused ? Theme.borderStrong
                      : (root.hovered && !root.readOnly ? Theme.borderStrong : Theme.borderSoft)
        clip: true

        Behavior on color {
            ColorAnimation { duration: Theme.motionFast }
        }

        Behavior on border.color {
            ColorAnimation { duration: Theme.motionFast }
        }

        // A linha do foco: cresce do centro para as pontas.
        Rectangle {
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            height: 2
            width: root.focused || root.error ? parent.width : 0
            color: root.error ? Theme.errorSoft : Theme.accent

            Behavior on width {
                NumberAnimation { duration: Theme.motionMedium; easing.type: Theme.easingStandard }
            }
        }
    }

    HoverHandler {
        id: hover

        cursorShape: Qt.IBeamCursor
    }

    KvIcon {
        id: leading

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + 2
        anchors.verticalCenter: parent.verticalCenter
        visible: root.iconName !== ""
        name: root.iconName === "" ? "search" : root.iconName
        size: 14
        active: root.focused
    }

    // O rotulo flutuante: no meio quando vazio, no alto e menor quando sobe.
    Text {
        id: floatingLabel

        x: textInput.x
        y: root.floated ? 5 : (root.height - height) / 2
        // A largura que sobra, desfeita a escala: o rotulo longo termina em
        // reticencias em vez de passar da borda.
        width: Math.max(0, (root.width - x - Theme.spacingMedium) / scale)
        elide: Text.ElideRight
        visible: root.hasLabel
        text: root.label
        color: root.error ? Theme.errorSoft : (root.focused ? Theme.accent : Theme.textMuted)
        font.family: Theme.uiFont
        font.pixelSize: root.pixelSize
        scale: root.floated ? 0.78 : 1
        transformOrigin: Item.TopLeft

        Behavior on y {
            NumberAnimation { duration: Theme.motionMedium; easing.type: Theme.easingStandard }
        }

        Behavior on scale {
            NumberAnimation { duration: Theme.motionMedium; easing.type: Theme.easingStandard }
        }

        Behavior on color {
            ColorAnimation { duration: Theme.motionFast }
        }
    }

    TextInput {
        id: textInput

        anchors.left: leading.visible ? leading.right : parent.left
        anchors.leftMargin: leading.visible ? Theme.spacingSmall : Theme.spacingMedium
        anchors.right: clearButton.left
        anchors.rightMargin: Theme.spacingXSmall
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.hasLabel ? 4 : 0
        height: root.hasLabel ? parent.height - 18 : parent.height
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

        // O clique DIREITO abre Recortar/Copiar/Colar/Selecionar tudo
        // (2026-10-04). Uma area SO' do botao direito, por cima da entrada: ela
        // fica com o aperto antes de a entrada mover o cursor e desfazer a
        // selecao (Copiar vinha apagado); o esquerdo passa direto.
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.RightButton
            cursorShape: Qt.IBeamCursor
            onPressed: (mouse) => {
                textInput.forceActiveFocus();
                const scene = textInput.mapToItem(null, mouse.x, mouse.y);
                TextMenuController.openFor(root, scene.x, scene.y, Clipboard.text() !== "");
            }
        }

        // O cursor ambar com piscar suave (KvTextCaret).
        cursorDelegate: KvTextCaret {
            input: root.input
        }

        // O placeholder: esmaece e desliza ao comecar a digitar. Com rotulo,
        // so' aparece com o rotulo ja' no alto.
        Text {
            anchors.fill: parent
            anchors.leftMargin: textInput.text === "" ? 0 : Theme.spacingSmall
            verticalAlignment: Text.AlignVCenter
            horizontalAlignment: textInput.horizontalAlignment
            opacity: textInput.text === "" && textInput.preeditText === "" && (!root.hasLabel || root.focused) ? 1 : 0
            text: root.placeholder
            color: Theme.textMuted
            elide: Text.ElideRight
            font.family: textInput.font.family
            font.pixelSize: textInput.font.pixelSize

            Behavior on opacity {
                NumberAnimation { duration: Theme.motionFast }
            }

            Behavior on anchors.leftMargin {
                NumberAnimation { duration: Theme.motionFast }
            }
        }
    }

    KvIconButton {
        id: clearButton

        anchors.right: parent.right
        anchors.rightMargin: root.clearable ? 3 : 0
        anchors.verticalCenter: textInput.verticalCenter
        readonly property bool shown: root.clearable && textInput.text !== "" && !root.readOnly
        width: root.clearable ? Math.min(22, root.height - 6) : 0
        height: width
        opacity: clearButton.shown ? 1 : 0
        enabled: clearButton.shown
        compact: true
        iconSize: 12
        iconName: "close"
        tooltip: qsTr("Limpar")
        focusOnClick: false
        onClicked: {
            root.clear();
            textInput.forceActiveFocus();
        }

        Behavior on opacity {
            NumberAnimation { duration: Theme.motionFast }
        }
    }
}

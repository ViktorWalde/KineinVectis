import QtQuick
import KineinVectis

// 16.5b: o tooltip aberto acompanha o texto do botao (o "Atualizar" que
// ficava com o "testa o endereco…" antigo); o TooltipController e o
// KvIconButton REAIS.
Item {
    id: root
    width: 400; height: 240
    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    KvIconButton { id: button; x: 40; y: 40; iconName: "refresh"; tooltip: "testa o endereço…" }
    KvIconButton { id: other; x: 140; y: 40; iconName: "close"; tooltip: "fechar" }

    property var temporaryButton: null
    Component { id: dynamicButton; KvButton { text: "Identificar"; tooltip: "Consulta a placa" } }
    Item { id: container; x: 40; y: 100; width: 100; height: 40 }

    function verifyLifetime() {
        temporaryButton = dynamicButton.createObject(container);
        TooltipController.showFor(temporaryButton, temporaryButton.tooltip, "bottom");
        check(TooltipController.shown, "controle dinâmico abre dica");
        container.visible = false;
        check(!TooltipController.shown, "ocultar ancestral encerra a dica");
        TooltipController.showFor(temporaryButton, temporaryButton.tooltip, "bottom");
        check(!TooltipController.shown, "dono oculto não reabre dica");
        container.visible = true;
        temporaryButton.enabled = false;
        TooltipController.showFor(temporaryButton, temporaryButton.tooltip, "bottom");
        check(TooltipController.shown, "desabilitar não oculta explicação");
        temporaryButton.destroy();
        Qt.callLater(verifyDestroyed);
    }
    function verifyDestroyed() {
        check(!TooltipController.shown && TooltipController.text === "", "destruir dono encerra dica e texto");
        check(TooltipController.owner === null, "destruir limpa referência do dono");
        temporaryButton = dynamicButton.createObject(container);
        TooltipController.showFor(temporaryButton, temporaryButton.tooltip, "bottom");
        TooltipController.showFor(other, other.tooltip, "right");
        TooltipController.hideFor(temporaryButton);
        TooltipController.hideFor(null);
        check(TooltipController.shown && TooltipController.owner === other, "saída atrasada do dono anterior não apaga dica atual");
        temporaryButton.destroy();
        Qt.callLater(verifyNewOwner);
    }
    function verifyNewOwner() {
        check(TooltipController.shown && TooltipController.owner === other, "destruir dono anterior preserva dica atual");
        other.visible = false;
        check(!TooltipController.shown, "ocultar próprio dono encerra a dica");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }

    Component.onCompleted: {
        TooltipController.showFor(button, button.tooltip, "bottom");
        button.tooltip = "Atualizar";
        root.check(TooltipController.shown && TooltipController.text === "Atualizar",
            "o texto acompanha: " + TooltipController.text);
        other.tooltip = "outro";
        root.check(TooltipController.text === "Atualizar", "o de outro botao nao mexe");
        button.tooltip = "";
        root.check(!TooltipController.shown, "vazio esconde");
        root.verifyLifetime();
    }
}

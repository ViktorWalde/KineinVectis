import QtQuick
import KineinVectis

// 16.5b: o tooltip aberto acompanha o texto do botao (o "Atualizar" que
// ficava com o "testa o endereco…" antigo); o TooltipController e o
// KvIconButton REAIS.
Item {
    id: root
    width: 200; height: 100
    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    KvIconButton { id: button; iconName: "refresh"; tooltip: "testa o endereço…" }
    KvIconButton { id: other; iconName: "close"; tooltip: "fechar" }

    Component.onCompleted: {
        TooltipController.showFor(button, button.tooltip, "bottom");
        button.tooltip = "Atualizar";
        root.check(TooltipController.shown && TooltipController.text === "Atualizar",
            "o texto acompanha: " + TooltipController.text);
        other.tooltip = "outro";
        root.check(TooltipController.text === "Atualizar", "o de outro botao nao mexe");
        button.tooltip = "";
        root.check(!TooltipController.shown, "vazio esconde");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}

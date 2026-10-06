import QtQuick
import KineinVectis

// O seletor emite a escolha e conserva o binding do dono.
// Teclado pula opcoes desligadas; nenhuma opcao habilitada nao emite.
Item {
    id: root
    width: 600
    height: 100
    property string choice: "a"
    property int requests: 0
    KvSegmentedControl {
        id: ctl
        width: parent.width
        current: root.choice
        options: [{value: "a", label: "Primeira"}, {value: "b", label: "Desligada", enabled: false},
                  {value: "c", label: "Terceira"}]
        onSelected: value => { root.requests += 1; root.choice = value; }
    }
    function check(ok, message) {
        if (!ok) console.error("FALHOU: " + message);
        return ok ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        ctl.step(1);
        failures += check(root.choice === "c" && root.requests === 1, "pula desligada");
        ctl.step(1);
        failures += check(root.choice === "a", "volta ao inicio");
        ctl.step(-1);
        failures += check(root.choice === "c", "seta esquerda");
        root.choice = "a";
        failures += check(ctl.current === "a", "binding continua vivo");
        ctl.options = [{value: "x", label: "X", enabled: false}];
        const count = root.requests;
        ctl.step(1);
        failures += check(root.requests === count, "todas desligadas");
        ctl.options = [];
        ctl.step(-1);
        failures += check(root.requests === count, "lista vazia");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}

// O menu existente e acessivel sem mouse; indisponiveis nao entram no ciclo.
import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property var actions: []
    property int dismissals: 0

    ProjectEntryMenuKeyboard {
        id: keyboard
        onActionRequested: index => root.actions.push(index)
        onDismissRequested: root.dismissals += 1
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            failures += 1;
        }
    }

    function key(code) { return {key: code}; }

    Component.onCompleted: {
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 1, "seta nao avancou");
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 4, "nao pulou executar/depurar ausentes");
        keyboard.singleSelection = false;
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 0, "nao pulou renomear/excluir em grupo");
        keyboard.handleKey(root.key(Qt.Key_Up));
        root.check(keyboard.currentAction === 1, "seta acima nao voltou pelo ciclo disponivel");
        keyboard.runnableScript = true;
        keyboard.debuggableScript = true;
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 2, "executar disponivel nao entrou no ciclo");
        keyboard.handleKey(root.key(Qt.Key_Return));
        root.check(root.actions.length === 1 && root.actions[0] === 2,
                   "Enter nao disparou acao atual");
        keyboard.handleKey(root.key(Qt.Key_Escape));
        root.check(root.dismissals === 1, "Escape nao fechou o menu");
        root.check(!keyboard.handleKey(root.key(Qt.Key_C)), "letra alheia foi consumida");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}

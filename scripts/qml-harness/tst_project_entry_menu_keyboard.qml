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
        root.check(keyboard.currentAction === 6, "grupo nao alcancou copiar apos renomear/excluir");
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 7, "grupo nao alcancou recortar");
        keyboard.currentAction = 1;
        keyboard.runnableScript = true;
        keyboard.debuggableScript = true;
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 2, "executar disponivel nao entrou no ciclo");
        keyboard.handleKey(root.key(Qt.Key_Return));
        root.check(root.actions.length === 1 && root.actions[0] === 2,
                   "Enter nao disparou acao atual");
        keyboard.singleSelection = true;
        keyboard.pasteAvailable = true;
        keyboard.currentAction = 5;
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 6, "menu nao alcancou copiar");
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 7, "menu nao alcancou recortar");
        keyboard.handleKey(root.key(Qt.Key_Down));
        keyboard.handleKey(root.key(Qt.Key_Return));
        root.check(keyboard.currentAction === 8 && root.actions[1] === 8,
                   "menu nao alcancou colar");
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 9, "menu nao alcancou caminho absoluto");
        keyboard.handleKey(root.key(Qt.Key_Down));
        keyboard.handleKey(root.key(Qt.Key_Return));
        root.check(keyboard.currentAction === 10 && root.actions[2] === 10,
                   "menu nao alcancou caminho relativo");
        keyboard.handleKey(root.key(Qt.Key_Down));
        root.check(keyboard.currentAction === 11,
                   "menu nao alcancou gerenciador de arquivos");
        keyboard.handleKey(root.key(Qt.Key_Down));
        keyboard.handleKey(root.key(Qt.Key_Return));
        root.check(keyboard.currentAction === 12 && root.actions[3] === 12,
                   "menu nao alcancou terminal nesta pasta");
        keyboard.handleKey(root.key(Qt.Key_Escape));
        root.check(root.dismissals === 1, "Escape nao fechou o menu");
        root.check(!keyboard.handleKey(root.key(Qt.Key_C)), "letra alheia foi consumida");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}

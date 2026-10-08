import QtQuick
import KineinVectis

// O token de comentario por linguagem (2026-10-08): o QML ganhou linguagem
// propria no realce (59 §2.4) e nao pode perder o `//` do Ctrl+/ que tinha
// quando caia nas regras de JavaScript.
Item {
    LanguageCommentRules {
        id: rules
    }

    Component.onCompleted: {
        let failures = 0;
        const check = (condition, label) => {
            if (!condition) {
                console.error("FALHOU: " + label);
                failures++;
            }
        };
        check(rules.tokenFor("qml") === "//", "qml comenta com //");
        check(rules.tokenFor("js") === "//", "js comenta com //");
        check(rules.tokenFor("python") === "#", "python comenta com #");
        check(rules.tokenFor("plain") === "", "linguagem sem token nao comenta");
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}

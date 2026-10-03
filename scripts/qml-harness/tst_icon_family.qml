import QtQuick
import KineinVectis
import "../../ui/qml/components/KvIconGlyphs.js" as Glyphs

// A familia de icones (0.3.9, KvIconGlyphs.js) e' SVG path no Canvas. Um path
// que o parser do Qt nao entende NAO da erro: o icone some calado. Aqui cada
// nome e' pintado num Canvas 24x24 e precisa acender pixels — nas duas
// versoes do Qt (o AppImage roda 6.4). Prova tambem que os nomes usados pelo
// trilho e pelo topo existem na familia.
Item {
    id: root

    width: 48
    height: 48

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Canvas {
        id: canvas

        width: 24
        height: 24

        onPaint: {
            let failures = 0;
            const context = getContext("2d");
            const names = Glyphs.names();
            failures += root.check(names.length >= 50, "familia com " + names.length + " nomes");
            for (const name of names) {
                context.reset();
                context.strokeStyle = "#ffffff";
                context.fillStyle = "#ffffff";
                context.lineWidth = 2;
                context.lineCap = "round";
                context.lineJoin = "round";
                const shape = Glyphs.shape(name);
                if (shape.stroke !== "") {
                    context.path = shape.stroke;
                    context.stroke();
                }
                if (shape.fill !== "") {
                    context.path = shape.fill;
                    context.fill();
                }
                const data = context.getImageData(0, 0, 24, 24).data;
                let lit = 0;
                for (let i = 3; i < data.length; i += 4) {
                    if (data[i] > 64) lit += 1;
                }
                // Um icone de verdade acende dezenas de pixels; o menor (o
                // "minimizar") passa de 20.
                failures += root.check(lit >= 20, name + " acendeu " + lit + " pixels");
            }
            for (const used of ["project", "terminal", "outline", "embedded", "database", "container",
                                "remote", "observability", "tools", "git", "run", "stop", "build",
                                "debug", "menu", "more", "pin"]) {
                failures += root.check(Glyphs.shape(used) !== null, "nome usado fora da familia: " + used);
            }
            failures += root.check(Glyphs.shape("tree-file-cpp") === null, "tipo de arquivo nao e' da familia");
            if (failures !== 0) console.error("FALHAS " + failures);
            Qt.exit(failures === 0 ? 0 : 1);
        }
    }
}

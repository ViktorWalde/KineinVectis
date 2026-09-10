pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O GRAFICO da trajetoria: y(t), e y'(t) quando a forma tem uma.
//
// `Canvas` e' raster 2D — sem GPU, sem `ShaderEffect`, sem `QRhi`. O invariante
// do `verificar-appimage.sh` fica intacto, e o precedente ja' existe no
// `KvIcon`. O `kinein-sim` com OpenGL so' entra quando a simulacao for CAMPO ou
// 3D; para curva sobre eixos, a UI desenha sozinha (`arquitetura/34` §10.2).
//
// O que ele NAO faz: nao calcula nada da fisica. Recebe a trilha ja' amostrada
// pelo core e a mapeia em pixels. Escalar dado para tela e' apresentacao;
// decidir a amostragem e' regra, e mora no core.
Item {
    id: root

    property var trail: []
    // Quando `true`, desenha tambem a derivada num segundo tracado.
    property bool showDerivative: true

    // --- a escala, separada do desenho para poder ser TESTADA ---------------
    //
    // Ela tem tres casos degenerados que um grafico ingenuo quebra: trilha
    // vazia, um ponto so', e curva CONSTANTE (onde `max - min` e' zero e a
    // divisao explode). Os tres aparecem de verdade: uma simulacao que diverge
    // no primeiro passo, e um oscilador com y(0)=0 e v(0)=0, que nao sai do
    // lugar.
    function extent(campo) {
        if (!root.trail || root.trail.length === 0) {
            return null;
        }
        let menor = Infinity;
        let maior = -Infinity;
        for (let i = 0; i < root.trail.length; ++i) {
            const v = root.trail[i][campo];
            if (v === undefined || v === null || !isFinite(v)) {
                continue;
            }
            if (v < menor) menor = v;
            if (v > maior) maior = v;
        }
        if (!isFinite(menor) || !isFinite(maior)) {
            return null;
        }
        if (maior - menor < 1e-15) {
            // Curva constante: abre uma janela em volta dela em vez de dividir
            // por zero. Sem isto o tracado sairia numa linha no topo ou some.
            const meio = Math.abs(menor) < 1e-15 ? 1 : Math.abs(menor);
            return { min: menor - meio * 0.5, max: maior + meio * 0.5 };
        }
        const folga = (maior - menor) * 0.08;
        return { min: menor - folga, max: maior + folga };
    }

    function mapX(t, limites, largura) {
        if (limites === null || limites.max - limites.min < 1e-300) {
            return largura / 2;
        }
        return ((t - limites.min) / (limites.max - limites.min)) * largura;
    }

    function mapY(v, limites, altura) {
        if (limites === null || limites.max - limites.min < 1e-300) {
            return altura / 2;
        }
        // Invertido: em tela, y cresce para baixo.
        return altura - ((v - limites.min) / (limites.max - limites.min)) * altura;
    }

    function fmt(x) {
        if (x === undefined || x === null || !isFinite(x)) {
            return "—";
        }
        const a = Math.abs(x);
        if (a !== 0 && (a < 1e-3 || a >= 1e5)) {
            return x.toExponential(2);
        }
        return x.toFixed(3);
    }

    readonly property var limitesT: root.extent("t")
    readonly property var limitesY: root.extent("y")
    readonly property bool temDerivada: {
        if (!root.trail || root.trail.length === 0) {
            return false;
        }
        const p = root.trail[0];
        return p.dy !== undefined && p.dy !== null;
    }

    implicitHeight: 200

    onTrailChanged: tela.requestPaint()
    onWidthChanged: tela.requestPaint()
    onHeightChanged: tela.requestPaint()

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
        border.color: Theme.borderSoft

        Canvas {
            id: tela

            anchors.fill: parent
            anchors.margins: 1
            renderStrategy: Canvas.Immediate

            onPaint: {
                const ctx = tela.getContext("2d");
                ctx.reset();
                const w = tela.width;
                const h = tela.height;
                if (w <= 0 || h <= 0) {
                    return;
                }

                const lt = root.limitesT;
                const ly = root.limitesY;
                if (lt === null || ly === null) {
                    return;
                }

                // A grade, para o olho medir.
                ctx.strokeStyle = Theme.borderSoft;
                ctx.lineWidth = 1;
                ctx.globalAlpha = 0.5;
                for (let g = 1; g < 4; ++g) {
                    const y = (h / 4) * g;
                    ctx.beginPath();
                    ctx.moveTo(0, y);
                    ctx.lineTo(w, y);
                    ctx.stroke();
                }
                ctx.globalAlpha = 1;

                // A linha do zero, quando ela esta' na janela: sem ela nao da'
                // para ver se a curva cruza ou so' se aproxima.
                if (ly.min < 0 && ly.max > 0) {
                    ctx.strokeStyle = Theme.textDisabled;
                    ctx.beginPath();
                    const zero = root.mapY(0, ly, h);
                    ctx.moveTo(0, zero);
                    ctx.lineTo(w, zero);
                    ctx.stroke();
                }

                // A derivada primeiro, por baixo.
                if (root.showDerivative && root.temDerivada) {
                    const ld = root.extent("dy");
                    if (ld !== null) {
                        ctx.strokeStyle = Theme.warningSoft;
                        ctx.lineWidth = 1;
                        ctx.globalAlpha = 0.7;
                        ctx.beginPath();
                        for (let i = 0; i < root.trail.length; ++i) {
                            const p = root.trail[i];
                            if (!isFinite(p.dy)) continue;
                            const x = root.mapX(p.t, lt, w);
                            const y = root.mapY(p.dy, ld, h);
                            if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
                        }
                        ctx.stroke();
                        ctx.globalAlpha = 1;
                    }
                }

                // A trajetoria.
                ctx.strokeStyle = Theme.accent;
                ctx.lineWidth = 1.75;
                ctx.beginPath();
                let comecou = false;
                for (let i = 0; i < root.trail.length; ++i) {
                    const p = root.trail[i];
                    if (!isFinite(p.y)) continue;
                    const x = root.mapX(p.t, lt, w);
                    const y = root.mapY(p.y, ly, h);
                    if (!comecou) { ctx.moveTo(x, y); comecou = true; }
                    else ctx.lineTo(x, y);
                }
                ctx.stroke();
            }
        }

        // Os limites em texto: um grafico sem escala nao e' medida, e' desenho.
        Text {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.margins: Theme.spacingXSmall
            text: root.limitesY === null ? "" : root.fmt(root.limitesY.max)
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus - 2
        }

        Text {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingXSmall
            text: root.limitesY === null ? "" : root.fmt(root.limitesY.min)
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus - 2
        }

        Text {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingXSmall
            text: root.limitesT === null ? "" : "t = " + root.fmt(root.limitesT.max)
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus - 2
        }

        Text {
            anchors.centerIn: parent
            visible: !root.trail || root.trail.length === 0
            text: qsTr("sem trajetória ainda")
            color: Theme.textDisabled
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }
}

pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O GRAFICO DA FORMA VETORIAL, em dois modos.
//
// `Canvas` e' raster 2D — sem GPU, sem `ShaderEffect`, sem `QRhi`. O invariante
// do `verificar-appimage.sh` fica intacto, e a trajetoria de uma orbita PLANA e'
// 2D de verdade: nao e' projecao de conveniencia. A vista 3D depende do
// `kinein-sim` e e' fatia propria (`arquitetura/34` §13.6).
//
//   COMPONENTES   n curvas sobre t. Serve a qualquer sistema
//   TRAJETORIA    um componente contra outro. O CONCEITO declara o par, e e' o
//                 modo em que a orbita que nao fecha aparece como o que ela e'
//
// O que ele NAO faz: nao calcula nada da fisica. Recebe a trilha ja' amostrada
// pelo core e a mapeia em pixels.
Item {
    id: root

    // Cada amostra e' { t, values: [n] }.
    property var trail: []
    // O par que a trajetoria desenha, ou `null` quando o conceito nao declara.
    property var plane: null
    // "components" ou "trajectory".
    property string mode: "components"
    // Os rotulos, na ordem do conceito.
    property var componentLabels: []

    // --- a escala, separada do desenho para poder ser TESTADA ---------------
    //
    // Os mesmos tres casos degenerados do `SimPlot2d`, e um QUARTO que so' a
    // forma vetorial tem: `values` com comprimento diferente entre amostras, ou
    // um indice que nao existe. Uma corrida recusada no meio deixa a trilha com
    // amostras curtas, e `values[3]` de um array de dois da' `undefined` — que
    // vira `NaN` na conta e some sem erro.
    //
    // QUAL LINHA DEFENDE, medido por mutacao em 2026-09-06 — e nao era a que
    // parecia:
    //
    //   teto de comprimento       REDUNDANTE. `values[3]` de um array de dois
    //   (`indice < length`)       ja' chega como `undefined`. Saiu do codigo
    //   guarda de `undefined`     NAO mata nenhuma mutacao. Em JS
    //                             `undefined < x` e `undefined > x` sao os DOIS
    //                             falsos, entao o valor e' pulado de qualquer
    //                             jeito. FICA mesmo assim, e a razao esta abaixo
    //   `!isFinite` nos EXTREMOS  E' ESTA. Sem ela, uma trilha em que o indice
    //                             nao existe em amostra NENHUMA devolve
    //                             `{min: Infinity, max: -Infinity}` em vez de
    //                             `null`, e o grafico desenha lixo
    //
    // O guarda de `undefined` fica porque a alternativa e' depender de como o
    // JS compara `undefined` — comportamento implicito, que e' exatamente o que
    // este projeto nao aceita como defesa. Ele nao esta' aqui por medo: esta'
    // aqui para o pulo ser LEGIVEL em vez de acidental.
    function extentOf(indice) {
        if (!root.trail || root.trail.length === 0) {
            return null;
        }
        let menor = Infinity;
        let maior = -Infinity;
        for (let i = 0; i < root.trail.length; ++i) {
            const amostra = root.trail[i];
            const v = indice < 0 ? amostra.t
                                 : (amostra.values ? amostra.values[indice] : undefined);
            if (v === undefined || v === null || !isFinite(v)) {
                continue;
            }
            if (v < menor) menor = v;
            if (v > maior) maior = v;
        }
        if (!isFinite(menor) || !isFinite(maior)) {
            return null;
        }
        // Curva CONSTANTE: `maior - menor` e' zero e a divisao explodiria. A
        // janela abre em vez de dividir por zero.
        if (maior - menor < 1e-15) {
            const folga = Math.max(Math.abs(maior) * 0.1, 1e-9);
            return { min: menor - folga, max: maior + folga };
        }
        return { min: menor, max: maior };
    }

    // O extent COMUM aos dois eixos da trajetoria.
    //
    // Sem isso um circulo vira elipse: escalar x e y por limites diferentes
    // deforma a figura, e a orbita — que e' o caso de uso da trajetoria —
    // deixaria de mostrar que fechou.
    function squareExtent(a, b) {
        if (a === null || b === null) {
            return null;
        }
        const centroA = (a.min + a.max) / 2;
        const centroB = (b.min + b.max) / 2;
        const lado = Math.max(a.max - a.min, b.max - b.min) / 2;
        return {
            x: { min: centroA - lado, max: centroA + lado },
            y: { min: centroB - lado, max: centroB + lado }
        };
    }

    function mapX(valor, limites, largura) {
        if (limites === null || !isFinite(valor)) {
            return largura / 2;
        }
        return (valor - limites.min) / (limites.max - limites.min) * largura;
    }

    function mapY(valor, limites, altura) {
        if (limites === null || !isFinite(valor)) {
            return altura / 2;
        }
        // O eixo da tela cresce para BAIXO, e o do grafico para cima.
        return altura - (valor - limites.min) / (limites.max - limites.min) * altura;
    }

    // A cor de cada componente, estavel entre redesenhos.
    function componentColor(indice) {
        const paleta = [Theme.accent, Theme.successSoft, Theme.infoSoft,
                        Theme.errorSoft, Theme.purpleOrbital, Theme.warningSoft];
        return paleta[indice % paleta.length];
    }

    implicitHeight: 220

    Canvas {
        id: tela

        anchors.fill: parent
        renderStrategy: Canvas.Immediate

        onPaint: {
            const ctx = getContext("2d");
            ctx.reset();
            ctx.fillStyle = Theme.surface2;
            ctx.fillRect(0, 0, width, height);
            if (!root.trail || root.trail.length === 0) {
                return;
            }
            if (root.mode === "trajectory" && root.plane !== null) {
                root.paintTrajectory(ctx);
            } else {
                root.paintComponents(ctx);
            }
        }
    }

    function paintComponents(ctx) {
        const tempo = extentOf(-1);
        const quantos = trail[0].values ? trail[0].values.length : 0;
        for (let c = 0; c < quantos; ++c) {
            const limites = extentOf(c);
            if (limites === null) {
                continue;
            }
            ctx.strokeStyle = componentColor(c);
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            for (let i = 0; i < trail.length; ++i) {
                const amostra = trail[i];
                if (!amostra.values || !isFinite(amostra.values[c])) {
                    continue;
                }
                const px = mapX(amostra.t, tempo, tela.width);
                const py = mapY(amostra.values[c], limites, tela.height);
                if (i === 0) ctx.moveTo(px, py);
                else ctx.lineTo(px, py);
            }
            ctx.stroke();
        }
    }

    function paintTrajectory(ctx) {
        const primeiro = plane.first;
        const segundo = plane.second;
        const quadrado = squareExtent(extentOf(primeiro), extentOf(segundo));
        if (quadrado === null) {
            return;
        }
        ctx.strokeStyle = Theme.accent;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        for (let i = 0; i < trail.length; ++i) {
            const amostra = trail[i];
            if (!amostra.values || !isFinite(amostra.values[primeiro])
                || !isFinite(amostra.values[segundo])) {
                continue;
            }
            const px = mapX(amostra.values[primeiro], quadrado.x, tela.width);
            const py = mapY(amostra.values[segundo], quadrado.y, tela.height);
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
        }
        ctx.stroke();
        // O ponto de partida, para a trajetoria dizer onde comecou.
        const inicio = trail[0];
        if (inicio.values && isFinite(inicio.values[primeiro])
            && isFinite(inicio.values[segundo])) {
            ctx.fillStyle = Theme.successSoft;
            ctx.beginPath();
            ctx.ellipse(mapX(inicio.values[primeiro], quadrado.x, tela.width) - 3,
                        mapY(inicio.values[segundo], quadrado.y, tela.height) - 3, 6, 6);
            ctx.fill();
        }
    }

    onTrailChanged: tela.requestPaint()
    onModeChanged: tela.requestPaint()
    onPlaneChanged: tela.requestPaint()
}

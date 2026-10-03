pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A ONDA da tela de boas-vindas (2026-10-03, pedido do autor: um degrade
// "suave, nao o sRGB" da paleta do icone, com "animacao de movimento de onda",
// visualmente impactante no primeiro contato e desligavel).
//
//   fundo     brandInk -> um tom mais quente (OKLab, de cima para baixo)
//   cristas   tres elipses largas embaixo: ardosia (a de tras), ambar
//             escuro, ambar (a da frente), cada uma com o degrade indo a
//             transparente — o arco de cima e' a crista da onda
//   movimento cada crista desliza de lado e sobe/desce num ciclo LENTO e
//             proprio (Theme.motionWave), com curva senoidal: paralaxe
//
// So' POSICAO das cristas muda: o degrade nao e' redesenhado a cada passo.
// `animated` falso (a preferencia, ou a tela fora de vista) para o relogio e
// congela tudo onde esta' — parado, a onda continua bonita; nada roda.
Item {
    id: root

    property bool animated: true
    // O relogio da onda, em ms. Ele anda a 20 passos por segundo (e nao a
    // cada quadro, 60 por segundo): a onda percorre poucos pixels por
    // segundo, e o passo mais curto so' gastava CPU — medido na tela real em
    // 2026-10-03: 10,8% de um nucleo com animacao a 60 quadros.
    property real phase: 0

    clip: true

    Timer {
        interval: Theme.motionWaveTick
        repeat: true
        running: root.animated
        onTriggered: root.phase += interval
    }

    // O fundo: a tinta do icone, mais quente embaixo — sete paradas ja'
    // misturadas em OKLab por scripts/gerar_degrade_boas_vindas.py (o QML so'
    // le as cores; nenhuma conta de cor roda aqui).
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.welcomeGradient[0] }
            GradientStop { position: 0.16; color: Theme.welcomeGradient[1] }
            GradientStop { position: 0.33; color: Theme.welcomeGradient[2] }
            GradientStop { position: 0.5; color: Theme.welcomeGradient[3] }
            GradientStop { position: 0.67; color: Theme.welcomeGradient[4] }
            GradientStop { position: 0.84; color: Theme.welcomeGradient[5] }
            GradientStop { position: 1.0; color: Theme.welcomeGradient[6] }
        }
    }

    // Uma CRISTA: um circulo esticado na horizontal (elipse) — a borda de
    // cima e' um ARCO, e tres arcos deslizando em ritmos diferentes sao as
    // colinas de uma onda. (Um retangulo de pontas redondas tem as bordas
    // longas RETAS: na tela, a primeira versao virou faixas inclinadas.)
    // A cor da crista e' UMA so', indo a transparente: so' a opacidade muda,
    // e duas paradas bastam (nao ha' mistura de cores a suavizar).
    component Crest: Rectangle {
        id: crest

        property color fromColor: Theme.brandSlate
        property real fromAlpha: 1

        property real stretch: 2.6
        property real driftX: 0.18
        property real driftY: 18
        property int period: Theme.motionWave
        property real baseX: 0
        property real baseY: 0

        width: root.height * 1.1
        height: width
        radius: width / 2
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.rgba(crest.fromColor.r, crest.fromColor.g, crest.fromColor.b, crest.fromAlpha) }
            GradientStop { position: 1.0; color: Qt.rgba(crest.fromColor.r, crest.fromColor.g, crest.fromColor.b, 0) }
        }
        transform: Scale {
            origin.x: crest.width / 2
            origin.y: crest.height / 2
            xScale: crest.stretch
        }

        // A posicao sai do relogio da onda (`phase`, em ms): vaivem senoidal
        // de lado e, num ritmo diferente, para cima e para baixo.
        x: crest.baseX + root.width * crest.driftX * (0.5 - 0.5 * Math.cos(Math.PI * root.phase / crest.period))
        y: crest.baseY - crest.driftY * (0.5 - 0.5 * Math.cos(Math.PI * root.phase / (crest.period * 0.7)))
    }

    // De tras para a frente: ardosia (mais alta), ambar escuro, ambar.
    Crest {
        fromColor: Theme.brandSlate
        fromAlpha: 0.30
        baseX: -root.width * 0.15
        baseY: root.height * 0.52
        driftX: 0.10
        driftY: 14
        period: Theme.motionWave * 1.4
    }

    Crest {
        fromColor: Theme.brandAmberDeep
        fromAlpha: 0.22
        stretch: 2.2
        baseX: root.width * 0.35
        baseY: root.height * 0.66
        driftX: -0.16
        driftY: 22
        period: Theme.motionWave
    }

    Crest {
        fromColor: Theme.brandAmber
        fromAlpha: 0.10
        stretch: 3.0
        baseX: root.width * 0.05
        baseY: root.height * 0.78
        driftX: 0.2
        driftY: 16
        period: Theme.motionWave * 0.8
    }
}

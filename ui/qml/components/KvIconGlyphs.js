// Glifos das FERRAMENTAS NATIVAS (banco, containers, observabilidade, remoto) — o mesmo grid
// 24x24 e o mesmo tracado do KvIcon, num modulo proprio para o KvIcon nao
// cruzar o limite de 300 linhas por causa de dois desenhos. O KvIcon chama
// `draw` no `default` do switch; quem nao esta' aqui volta `false` e o KvIcon
// desenha o "file" de sempre.
//
// Nenhum destes e' uma marca registrada: o container e' uma caixa com fita, a
// observabilidade e' um traco de pulso num mostrador, o banco e' o cilindro
// classico de disco. Cor e' do KvIcon (Theme).
.pragma library

function draw(name, context, line, node) {
    switch (name) {
    case "pin":
        // alfinete: cabeca, corpo e ponta (fixar uma area no trilho)
        line(context, 9, 4, 15, 4);
        line(context, 10, 4, 10, 11);
        line(context, 14, 4, 14, 11);
        line(context, 7, 11, 17, 11);
        line(context, 12, 11, 12, 20);
        return true;
    case "eye":
        // olho aberto: a area aparece
        context.moveTo(3, 12);
        context.quadraticCurveTo(12, 3, 21, 12);
        context.quadraticCurveTo(12, 21, 3, 12);
        context.moveTo(14.5, 12);
        context.arc(12, 12, 2.5, 0, Math.PI * 2, false);
        return true;
    case "eye-off":
        // olho riscado: a area esta' oculta
        context.moveTo(3, 12);
        context.quadraticCurveTo(12, 3, 21, 12);
        context.quadraticCurveTo(12, 21, 3, 12);
        line(context, 4, 20, 20, 4);
        return true;
    case "more":
        // tres pontos: o "Mais" do trilho (as areas fora dele)
        context.moveTo(7, 12);
        context.arc(6, 12, 1, 0, Math.PI * 2, false);
        context.moveTo(13, 12);
        context.arc(12, 12, 1, 0, Math.PI * 2, false);
        context.moveTo(19, 12);
        context.arc(18, 12, 1, 0, Math.PI * 2, false);
        return true;
    case "home":
        // casa: telhado, paredes e porta (o "Inicio" do seletor de pastas)
        context.moveTo(3, 11);
        context.lineTo(12, 4);
        context.lineTo(21, 11);
        context.moveTo(5.5, 9.5);
        context.lineTo(5.5, 20);
        context.lineTo(18.5, 20);
        context.lineTo(18.5, 9.5);
        context.moveTo(10, 20);
        context.lineTo(10, 15);
        context.lineTo(14, 15);
        context.lineTo(14, 20);
        return true;
    case "desktop":
        // monitor no pe': a area de trabalho
        context.moveTo(3, 5);
        context.lineTo(21, 5);
        context.lineTo(21, 16);
        context.lineTo(3, 16);
        context.closePath();
        line(context, 12, 16, 12, 20);
        line(context, 8, 20, 16, 20);
        return true;
    case "documents":
        // folha com a orelha dobrada e duas linhas de texto
        context.moveTo(6, 3);
        context.lineTo(14, 3);
        context.lineTo(19, 8);
        context.lineTo(19, 21);
        context.lineTo(6, 21);
        context.closePath();
        context.moveTo(14, 3);
        context.lineTo(14, 8);
        context.lineTo(19, 8);
        line(context, 9, 13, 16, 13);
        line(context, 9, 17, 16, 17);
        return true;
    case "download":
        // seta descendo para a bandeja
        line(context, 12, 4, 12, 15);
        context.moveTo(7.5, 10.5);
        context.lineTo(12, 15);
        context.lineTo(16.5, 10.5);
        line(context, 5, 19, 19, 19);
        return true;
    case "drive":
        // disco: a raiz do sistema de arquivos
        context.moveTo(3, 13);
        context.lineTo(21, 13);
        context.lineTo(21, 19);
        context.lineTo(3, 19);
        context.closePath();
        context.moveTo(5, 13);
        context.lineTo(7.5, 6);
        context.lineTo(16.5, 6);
        context.lineTo(19, 13);
        node(context, 17, 16, 1);
        return true;
    case "recent":
        // relogio: projetos abertos antes
        context.moveTo(20, 12);
        context.arc(12, 12, 8, 0, Math.PI * 2, false);
        context.moveTo(12, 7);
        context.lineTo(12, 12);
        context.lineTo(15.5, 14);
        return true;
    case "cpu":
        // chip com pinos: a toolchain efetiva (o chip de contexto do cabecalho)
        context.moveTo(7, 7);
        context.lineTo(17, 7);
        context.lineTo(17, 17);
        context.lineTo(7, 17);
        context.closePath();
        context.moveTo(10, 10);
        context.lineTo(14, 10);
        context.lineTo(14, 14);
        context.lineTo(10, 14);
        context.closePath();
        line(context, 10, 4, 10, 7);
        line(context, 14, 4, 14, 7);
        line(context, 10, 17, 10, 20);
        line(context, 14, 17, 14, 20);
        line(context, 4, 10, 7, 10);
        line(context, 4, 14, 7, 14);
        line(context, 17, 10, 20, 10);
        line(context, 17, 14, 20, 14);
        return true;
    case "add":
        // o "+" de criar (o "Criar Projeto" da tela inicial, 2026-10-01)
        line(context, 12, 5, 12, 19);
        line(context, 5, 12, 19, 12);
        return true;
    case "database":
        // o cilindro: tampa eliptica, dois flancos, duas cintas e o fundo.
        // As elipses sao arcos sob escala vertical; a escala e' desfeita antes
        // do stroke (quem traca e' o KvIcon), entao a linha fica na espessura.
        context.save();
        context.scale(1, 0.4);
        context.moveTo(19, 6 / 0.4);
        context.arc(12, 6 / 0.4, 7, 0, Math.PI * 2, false);
        context.moveTo(19, 12 / 0.4);
        context.arc(12, 12 / 0.4, 7, 0, Math.PI, false);
        context.moveTo(19, 18 / 0.4);
        context.arc(12, 18 / 0.4, 7, 0, Math.PI, false);
        context.restore();
        line(context, 5, 6, 5, 18);
        line(context, 19, 6, 19, 18);
        return true;
    case "container":
        // caixa isometrica: topo, frente, lado; a fita ao meio
        context.moveTo(4, 8);
        context.lineTo(12, 4);
        context.lineTo(20, 8);
        context.lineTo(12, 12);
        context.closePath();
        line(context, 4, 8, 4, 16);
        line(context, 4, 16, 12, 20);
        line(context, 12, 20, 20, 16);
        line(context, 20, 16, 20, 8);
        line(context, 12, 12, 12, 20);
        return true;
    case "observability":
        // mostrador aberto embaixo, com o pulso atravessando
        context.arc(12, 13, 8, Math.PI * 0.85, Math.PI * 2.15, false);
        context.moveTo(5, 13);
        context.lineTo(9, 13);
        context.lineTo(11, 8);
        context.lineTo(13, 18);
        context.lineTo(15, 13);
        context.lineTo(19, 13);
        return true;
    case "embedded":
        // o chip: o encapsulado, o die ao centro e tres pinos por lado
        context.moveTo(7, 7);
        context.lineTo(17, 7);
        context.lineTo(17, 17);
        context.lineTo(7, 17);
        context.closePath();
        context.moveTo(10, 10);
        context.lineTo(14, 10);
        context.lineTo(14, 14);
        context.lineTo(10, 14);
        context.closePath();
        for (let i = 0; i < 3; i++) {
            const p = 9 + i * 3;
            line(context, p, 3, p, 7);
            line(context, p, 17, p, 21);
            line(context, 3, p, 7, p);
            line(context, 17, p, 21, p);
        }
        return true;
    case "remote":
        // duas maquinas ligadas: a daqui, o salto e a de la'. As caixas tem a
        // ALTURA do grid (como o cilindro e o chip) — na primeira versao eram
        // 7x7 e o icone aparecia leve demais ao lado dos vizinhos no trilho.
        // O vao entre os tracos da ligacao e' o salto SSH: nao e' linha
        // continua porque a maquina do outro lado nao esta' aqui.
        context.moveTo(2, 5);
        context.lineTo(9, 5);
        context.lineTo(9, 19);
        context.lineTo(2, 19);
        context.closePath();
        context.moveTo(15, 5);
        context.lineTo(22, 5);
        context.lineTo(22, 19);
        context.lineTo(15, 19);
        context.closePath();
        line(context, 9, 12, 11, 12);
        line(context, 13, 12, 15, 12);
        node(context, 12, 12, 1);
        return true;
    default:
        return false;
    }
}

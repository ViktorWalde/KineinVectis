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

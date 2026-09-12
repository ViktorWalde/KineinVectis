// Glifos das FERRAMENTAS NATIVAS (containers, observabilidade) — o mesmo grid
// 24x24 e o mesmo tracado do KvIcon, num modulo proprio para o KvIcon nao
// cruzar o limite de 300 linhas por causa de dois desenhos. O KvIcon chama
// `draw` no `default` do switch; quem nao esta' aqui volta `false` e o KvIcon
// desenha o "file" de sempre.
//
// Nenhum destes e' uma marca registrada: o container e' uma caixa com fita, a
// observabilidade e' um traco de pulso num mostrador. Cor e' do KvIcon (Theme).
.pragma library

function draw(name, context, line, node) {
    switch (name) {
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
    default:
        return false;
    }
}

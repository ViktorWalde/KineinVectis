// Corpus do gate de duplicação (P20), forma correta c03: o serializador da
// resposta, com o mesmo esqueleto do da requisição e outros campos. O que se
// repete de verdade (o escape) já está em append_escaped.
#include "corpus/protocol/messages.hpp"

#include <string>

namespace corpus::protocol {

std::string encode_response(const Response& response) {
    std::string out{};
    out.reserve(48 + response.error.size());
    out.append("{\"id\":");
    out.append(std::to_string(response.id));
    out.append(",\"ok\":");
    out.append(response.outcome == Outcome::success ? "true" : "false");
    out.append(",\"error\":\"");
    append_escaped(out, response.error);
    out.append("\"}");
    return out;
}

}  // namespace corpus::protocol

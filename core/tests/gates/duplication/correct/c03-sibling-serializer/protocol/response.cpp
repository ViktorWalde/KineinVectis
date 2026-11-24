// Corpus of gate.duplication: correct form c03, the response serializer, with the same skeleton as
// the request's and other fields.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
//
// What truly repeats (the escaping) lives in append_escaped.
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

} // namespace corpus::protocol

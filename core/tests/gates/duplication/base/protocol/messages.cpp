// Corpus of gate.duplication: the protocol module of the base tree.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
#include "corpus/protocol/messages.hpp"

#include <string>
#include <string_view>

namespace corpus::protocol {

std::string_view method_name(Method method) {
    switch (method) {
    case Method::initialize:
        return "initialize";
    case Method::open_document:
        return "textDocument/didOpen";
    case Method::close_document:
        return "textDocument/didClose";
    case Method::shutdown:
        return "shutdown";
    }
    return "unknown";
}

void append_escaped(std::string& out, std::string_view text) {
    for (const char character : text) {
        switch (character) {
        case '"':
            out.append("\\\"");
            break;
        case '\\':
            out.append("\\\\");
            break;
        case '\n':
            out.append("\\n");
            break;
        case '\t':
            out.append("\\t");
            break;
        default:
            out.push_back(character);
            break;
        }
    }
}

std::string encode_request(const Request& request) {
    std::string out{};
    out.reserve(64 + request.document.size());
    out.append("{\"id\":");
    out.append(std::to_string(request.id));
    out.append(",\"method\":\"");
    out.append(method_name(request.method));
    out.append("\",\"document\":\"");
    append_escaped(out, request.document);
    out.append("\"}");
    return out;
}

} // namespace corpus::protocol

// Corpus of gate.duplication: defect d11, encode_request copied, calling another function with the
// same signature and with another literal.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/protocol/messages.hpp"

#include <array>
#include <cstddef>
#include <string>
#include <string_view>

namespace corpus::protocol {

std::string_view method_label(Method method) {
    constexpr std::array<std::string_view, 4> labels{"init", "open", "close", "exit"};
    return labels.at(static_cast<std::size_t>(method));
}

std::string encode_notification(const Request& request) { // reject: encode_request
    std::string out{};
    out.reserve(64 + request.document.size());
    out.append("{\"seq\":");
    out.append(std::to_string(request.id));
    out.append(",\"method\":\"");
    out.append(method_label(request.method));
    out.append("\",\"document\":\"");
    append_escaped(out, request.document);
    out.append("\"}");
    return out;
}

} // namespace corpus::protocol

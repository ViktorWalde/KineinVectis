// Corpus of gate.duplication: correct form c12, two std::visit with the same shape and different
// actions.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include <cstddef>
#include <string>
#include <variant>

namespace corpus::protocol {

struct Opened {
    std::string uri{};
};
struct Edited {
    std::string uri{};
    std::size_t bytes{};
};
struct Closed {
    std::string uri{};
};
using Event = std::variant<Opened, Edited, Closed>;

template <typename... Handlers>
struct Overloaded : Handlers... {
    using Handlers::operator()...;
};

std::string event_uri(const Event& event) {
    return std::visit(Overloaded{
                          [](const Opened& opened) { return opened.uri; },
                          [](const Edited& edited) { return edited.uri; },
                          [](const Closed& closed) { return closed.uri; },
                      },
                      event);
}

std::size_t event_weight(const Event& event) {
    return std::visit(Overloaded{
                          [](const Opened& opened) { return opened.uri.size(); },
                          [](const Edited& edited) { return edited.bytes; },
                          [](const Closed&) { return std::size_t{1}; },
                      },
                      event);
}

} // namespace corpus::protocol

// Corpus of gate.duplication: correct form c02, another enum table with the shape of method_name
// and severity_name.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
//
// The data change; the logic is the switch.
#include "corpus/protocol/messages.hpp"

#include <string_view>

namespace corpus::protocol {

enum class Category { lifecycle, document };

std::string_view outcome_name(Outcome outcome) {
    switch (outcome) {
    case Outcome::success:
        return "success";
    case Outcome::failure:
        return "failure";
    }
    return "unknown";
}

Category method_category(Method method) {
    switch (method) {
    case Method::initialize:
        return Category::lifecycle;
    case Method::open_document:
        return Category::document;
    case Method::close_document:
        return Category::document;
    case Method::shutdown:
        return Category::lifecycle;
    }
    return Category::lifecycle;
}

} // namespace corpus::protocol

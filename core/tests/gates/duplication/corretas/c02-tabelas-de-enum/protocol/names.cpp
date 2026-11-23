// Corpus do gate de duplicação (P20), forma correta c02: outra tabela de
// enum com a forma de method_name e severity_name. Os dados mudam, a lógica é
// o switch.
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

}  // namespace corpus::protocol

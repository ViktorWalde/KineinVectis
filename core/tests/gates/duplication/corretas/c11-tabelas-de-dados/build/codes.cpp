// Corpus do gate de duplicação (P20), forma correta c11: duas tabelas de
// dados com a mesma forma, consultadas por uma função só.
#include <array>
#include <cstddef>
#include <optional>
#include <span>
#include <string_view>

namespace corpus::build {

struct Code {
    std::string_view name{};
    int value{};
};

std::optional<int> find_code(std::span<const Code> table, std::string_view name) {
    for (const Code& entry : table) {
        if (entry.name == name) {
            return entry.value;
        }
    }
    return std::nullopt;
}

std::optional<int> signal_number(std::string_view name) {
    static constexpr std::array<Code, 6> signals{{
        {.name = "SIGHUP", .value = 1},
        {.name = "SIGINT", .value = 2},
        {.name = "SIGQUIT", .value = 3},
        {.name = "SIGKILL", .value = 9},
        {.name = "SIGSEGV", .value = 11},
        {.name = "SIGTERM", .value = 15},
    }};
    return find_code(signals, name);
}

std::optional<int> exit_status(std::string_view name) {
    static constexpr std::array<Code, 6> statuses{{
        {.name = "ok", .value = 0},
        {.name = "usage", .value = 64},
        {.name = "data", .value = 65},
        {.name = "input", .value = 66},
        {.name = "software", .value = 70},
        {.name = "config", .value = 78},
    }};
    return find_code(statuses, name);
}

}  // namespace corpus::build

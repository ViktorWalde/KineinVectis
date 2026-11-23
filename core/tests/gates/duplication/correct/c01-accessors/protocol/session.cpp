// Corpus of gate.duplication: correct form c01, one-line accessors, with the same shape as those of
// Document.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include <cstdint>
#include <string>
#include <utility>

namespace corpus::protocol {

class Session {
public:
    explicit Session(std::string name) : name_{std::move(name)} {}

    [[nodiscard]] const std::string& name() const noexcept { return name_; }
    [[nodiscard]] const std::string& root() const noexcept { return root_; }
    [[nodiscard]] std::uint64_t requests() const noexcept { return requests_; }
    [[nodiscard]] std::uint64_t failures() const noexcept { return failures_; }
    void set_root(std::string root) { root_ = std::move(root); }
    void count_request() noexcept { ++requests_; }
    void count_failure() noexcept { ++failures_; }

private:
    std::string name_{};
    std::string root_{};
    std::uint64_t requests_{0};
    std::uint64_t failures_{0};
};

} // namespace corpus::protocol

// Corpus of gate.duplication: the protocol module of the base tree.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
#ifndef CORPUS_PROTOCOL_MESSAGES_HPP
#define CORPUS_PROTOCOL_MESSAGES_HPP

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>
#include <utility>

namespace corpus::protocol {

enum class Method { initialize, open_document, close_document, shutdown };
enum class Outcome { success, failure };

struct Request {
    std::uint64_t id{};
    Method method{};
    std::string document{};
};

struct Response {
    std::uint64_t id{};
    Outcome outcome{};
    std::string error{};
};

[[nodiscard]] std::string_view method_name(Method method);
void append_escaped(std::string& out, std::string_view text);
[[nodiscard]] std::string encode_request(const Request& request);

class Document {
public:
    Document(std::string uri, std::string text) : uri_{std::move(uri)}, text_{std::move(text)} {}

    [[nodiscard]] const std::string& uri() const noexcept { return uri_; }
    [[nodiscard]] const std::string& text() const noexcept { return text_; }
    [[nodiscard]] std::uint64_t version() const noexcept { return version_; }

    void replace(std::size_t offset, std::size_t length, std::string_view replacement) {
        const auto start = std::min(offset, text_.size());
        const auto count = std::min(length, text_.size() - start);
        text_.replace(start, count, replacement);
        ++version_;
    }

private:
    std::string uri_{};
    std::string text_{};
    std::uint64_t version_{0};
};

} // namespace corpus::protocol

#endif

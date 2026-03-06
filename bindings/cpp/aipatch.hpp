#pragma once

#include <string>
#include <string_view>
#include <utility>

extern "C" {
#include "aipatch.h"
}

namespace aipatch {

class Result {
public:
    Result() noexcept : raw_{AIPATCH_OK, nullptr, 0} {}
    explicit Result(aipatch_result raw) noexcept : raw_(raw) {}

    Result(const Result&) = delete;
    Result& operator=(const Result&) = delete;

    Result(Result&& other) noexcept : raw_(other.release()) {}

    Result& operator=(Result&& other) noexcept {
        if (this != &other) {
            aipatch_result_free(&raw_);
            raw_ = other.release();
        }
        return *this;
    }

    ~Result() {
        aipatch_result_free(&raw_);
    }

    bool ok() const noexcept {
        return raw_.code == AIPATCH_OK;
    }

    int code() const noexcept {
        return raw_.code;
    }

    std::string_view message() const noexcept {
        if (raw_.message == nullptr) {
            return {};
        }
        return std::string_view(raw_.message, raw_.message_len);
    }

    const aipatch_result& raw() const noexcept {
        return raw_;
    }

private:
    aipatch_result release() noexcept {
        aipatch_result released = raw_;
        raw_.code = AIPATCH_OK;
        raw_.message = nullptr;
        raw_.message_len = 0;
        return released;
    }

    aipatch_result raw_;
};

struct CallResult {
    int abi_status = 0;
    Result result;

    bool abi_ok() const noexcept {
        return abi_status == 0;
    }

    bool ok() const noexcept {
        return abi_ok() && result.ok();
    }
};

inline CallResult check(std::string_view patch, std::string_view root_dir) {
    aipatch_result raw{AIPATCH_OK, nullptr, 0};
    const int abi_status = aipatch_check(
        patch.data(),
        patch.size(),
        root_dir.data(),
        root_dir.size(),
        &raw
    );
    return CallResult{abi_status, Result(raw)};
}

inline CallResult apply(std::string_view patch, std::string_view root_dir) {
    aipatch_result raw{AIPATCH_OK, nullptr, 0};
    const int abi_status = aipatch_apply(
        patch.data(),
        patch.size(),
        root_dir.data(),
        root_dir.size(),
        &raw
    );
    return CallResult{abi_status, Result(raw)};
}

inline std::string_view version() noexcept {
    return std::string_view(aipatch_version());
}

inline int abi_version() noexcept {
    return aipatch_abi_version();
}

} // namespace aipatch

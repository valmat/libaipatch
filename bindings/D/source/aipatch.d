module aipatch;

import std.string : fromStringz;

extern(C) {
    struct aipatch_result {
        int code;
        char* message;
        size_t message_len;
    }

    enum AIPATCH_OK = 0;
    enum AIPATCH_INVALID_ARGUMENT = 1;
    enum AIPATCH_PARSE_ERROR = 2;
    enum AIPATCH_IO_ERROR = 3;
    enum AIPATCH_PATCH_CONFLICT = 4;
    enum AIPATCH_PATH_VIOLATION = 5;
    enum AIPATCH_UNSUPPORTED = 6;
    enum AIPATCH_INTERNAL_ERROR = 7;

    int aipatch_check(const(char)*, size_t, const(char)*, size_t, aipatch_result*);

    int aipatch_apply(const(char)*, size_t, const(char)*, size_t, aipatch_result*);

    void aipatch_result_free(aipatch_result* result);
    const(char)* aipatch_version();
    int aipatch_abi_version();
}

final class OperationResult {
    private int abiStatus_;
    private aipatch_result raw_;

    this(int abiStatus, aipatch_result raw) {
        abiStatus_ = abiStatus;
        raw_ = raw;
    }

    ~this() {
        aipatch_result_free(&raw_);
    }

    @property int abiStatus() const {
        return abiStatus_;
    }

    @property bool abiOk() const {
        return abiStatus_ == 0;
    }

    @property bool ok() const {
        return abiOk && raw_.code == AIPATCH_OK;
    }

    @property int code() const {
        return raw_.code;
    }

    @property string message() const {
        if (raw_.message is null) {
            return "";
        }
        return cast(string) raw_.message[0 .. raw_.message_len].idup;
    }
}

OperationResult check(string patch, string rootDir) {
    aipatch_result raw = aipatch_result(0, null, 0);
    const int abiStatus = aipatch_check(patch.ptr, patch.length, rootDir.ptr, rootDir.length, &raw);
    return new OperationResult(abiStatus, raw);
}

OperationResult apply(string patch, string rootDir) {
    aipatch_result raw = aipatch_result(0, null, 0);
    const int abiStatus = aipatch_apply(patch.ptr, patch.length, rootDir.ptr, rootDir.length, &raw);
    return new OperationResult(abiStatus, raw);
}

string libraryVersion() {
    return fromStringz(aipatch_version()).idup;
}

int abiVersion() {
    return aipatch_abi_version();
}

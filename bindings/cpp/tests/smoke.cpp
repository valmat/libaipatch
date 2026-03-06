#include "aipatch.hpp"

#include <assert.h>
#include <fstream>
#include <string>

#include <stdlib.h>

int main() {
    char dir_template[] = "/tmp/libaipatch-cpp-test-XXXXXX";
    const char* root_dir = mkdtemp(dir_template);
    assert(root_dir != nullptr);

    const std::string add_patch =
        "*** Begin Patch\n"
        "*** Add File: smoke.txt\n"
        "+from cpp smoke test\n"
        "*** End Patch\n";

    auto check_result = aipatch::check(add_patch, root_dir);
    assert(check_result.abi_ok());
    assert(check_result.result.ok());
    assert(check_result.result.message().empty());

    auto apply_result = aipatch::apply(add_patch, root_dir);
    assert(apply_result.abi_ok());
    assert(apply_result.result.ok());
    assert(!apply_result.result.message().empty());

    std::ifstream file(std::string(root_dir) + "/smoke.txt");
    std::string contents;
    std::getline(file, contents);
    assert(contents == "from cpp smoke test");

    const std::string bad_patch = "bad patch";
    auto bad_result = aipatch::check(bad_patch, root_dir);
    assert(bad_result.abi_ok());
    assert(!bad_result.result.ok());
    assert(!bad_result.result.message().empty());

    assert(!aipatch::version().empty());
    assert(aipatch::abi_version() == 1);
    return 0;
}

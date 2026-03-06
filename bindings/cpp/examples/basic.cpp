#include "aipatch.hpp"

#include <fstream>
#include <iostream>
#include <string>

#include <stdlib.h>

int main() {
    char dir_template[] = "/tmp/libaipatch-cpp-example-XXXXXX";
    const char* root_dir = mkdtemp(dir_template);
    if (root_dir == nullptr) {
        std::cerr << "failed to create temp directory\n";
        return 1;
    }

    const std::string patch =
        "*** Begin Patch\n"
        "*** Add File: hello.txt\n"
        "+hello from C++ binding\n"
        "*** End Patch\n";

    auto check_result = aipatch::check(patch, root_dir);
    if (!check_result.abi_ok()) {
        std::cerr << "ABI failure during check: " << check_result.abi_status << "\n";
        return 1;
    }
    if (!check_result.result.ok()) {
        std::cerr << "check failed: " << check_result.result.message() << "\n";
        return 1;
    }

    auto apply_result = aipatch::apply(patch, root_dir);
    if (!apply_result.abi_ok()) {
        std::cerr << "ABI failure during apply: " << apply_result.abi_status << "\n";
        return 1;
    }
    if (!apply_result.result.ok()) {
        std::cerr << "apply failed: " << apply_result.result.message() << "\n";
        return 1;
    }

    std::ifstream file(std::string(root_dir) + "/hello.txt");
    std::string contents;
    std::getline(file, contents);

    std::cout << "libaipatch version: " << aipatch::version() << "\n";
    std::cout << "apply summary:\n" << apply_result.result.message();
    std::cout << "file contents: " << contents << "\n";
    return 0;
}

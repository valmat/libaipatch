module basic;

import aipatch;
import core.sys.posix.unistd : getpid;
import std.file : mkdirRecurse, readText, rmdirRecurse;
import std.path : buildPath;
import std.stdio : writeln, stderr;
import std.string : chomp;
import std.conv : to;

int main() {
    const rootDir = buildPath("/tmp", "libaipatch-d-example-" ~ to!string(getpid()));
    mkdirRecurse(rootDir);
    scope(exit) rmdirRecurse(rootDir);

    const patch =
        "*** Begin Patch\n" ~
        "*** Add File: hello.txt\n" ~
        "+hello from D binding\n" ~
        "*** End Patch\n";

    auto checkResult = check(patch, rootDir);
    if (!checkResult.abiOk) {
        stderr.writeln("ABI failure during check: ", checkResult.abiStatus);
        return 1;
    }
    if (!checkResult.ok) {
        stderr.writeln("check failed: ", checkResult.message);
        return 1;
    }

    auto applyResult = apply(patch, rootDir);
    if (!applyResult.abiOk) {
        stderr.writeln("ABI failure during apply: ", applyResult.abiStatus);
        return 1;
    }
    if (!applyResult.ok) {
        stderr.writeln("apply failed: ", applyResult.message);
        return 1;
    }

    const contents = readText(buildPath(rootDir, "hello.txt")).chomp;
    writeln("libaipatch version: ", libraryVersion());
    writeln("apply summary:\n", applyResult.message);
    writeln("file contents: ", contents);
    return 0;
}

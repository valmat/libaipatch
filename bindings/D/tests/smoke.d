module smoke;

import aipatch;
import core.sys.posix.unistd : getpid;
import std.conv : to;
import std.file : mkdirRecurse, readText, rmdirRecurse;
import std.path : buildPath;
import std.string : chomp;

int main() {
    const rootDir = buildPath("/tmp", "libaipatch-d-test-" ~ to!string(getpid()));
    mkdirRecurse(rootDir);
    scope(exit) rmdirRecurse(rootDir);

    const addPatch =
        "*** Begin Patch\n" ~
        "*** Add File: smoke.txt\n" ~
        "+from d smoke test\n" ~
        "*** End Patch\n";

    auto checkResult = check(addPatch, rootDir);
    assert(checkResult.abiOk);
    assert(checkResult.ok);
    assert(checkResult.message.length == 0);

    auto applyResult = apply(addPatch, rootDir);
    assert(applyResult.abiOk);
    assert(applyResult.ok);
    assert(applyResult.message.length > 0);

    const contents = readText(buildPath(rootDir, "smoke.txt")).chomp;
    assert(contents == "from d smoke test");

    auto badResult = check("bad patch", rootDir);
    assert(badResult.abiOk);
    assert(!badResult.ok);
    assert(badResult.message.length > 0);

    assert(libraryVersion().length > 0);
    assert(abiVersion() == 1);
    return 0;
}

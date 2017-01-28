#! /usr/bin/env python3

"""
Simplistic performance tester.
Removes the max and min time. Finds average of the rest.
"""

import subprocess as sp
from time import time

count = 7
folder = "/usr"

def test(tree_cmd):
    times = []
    cmd = "{} {} -n > /dev/null".format(tree_cmd, folder)
    print()
    print(tree_cmd)
    print("=" * len(tree_cmd))

    for i in range(count):
        t1 = time()
        sp.check_call(cmd, shell=True)
        dt = time() -t1
        times.append(dt)
        print("  ", dt)

    times.remove(min(times))
    times.remove(max(times))
    _total = sum(times)
    _avg = sum(times)/len(times)
    _min = min(times)
    _max = max(times)
    print("")
    print("avg: {}, min: {}, max: {}, total: {}".format(_avg, _min, _max, _total))


start_time = time()
print("Compiling")
sp.check_call("cargo build --release", shell=True)

test("target/release/tree-rs")
test("~/.cargo/bin/tree-rs")
test("tree")

print("finished in ", time() - start_time)
#! /usr/bin/env python3

"""
Simplistic performance tester.
Removes the max and min time. Finds average of the rest.
"""

import subprocess as sp
from time import time

COUNT = 7
FOLDER = "/usr"

def test(tree_cmd):
    times = []
    cmd = "{} {} -n > /dev/null".format(tree_cmd, FOLDER)
    print()
    print(tree_cmd)
    print("=" * len(tree_cmd))

    for i in range(COUNT):
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

    return {
        "avg": _avg,
        "min": _min,
        "max": _max,
    }


start_time = time()
print("Compiling")
sp.check_call("cargo build --release", shell=True)

tree_rs = test("target/release/tree-rs")
tree = test("tree")

print("")
print("Perf tree-rs/tree:", tree_rs["avg"]/tree["avg"])
print("Benchmarking finished in ", time() - start_time)

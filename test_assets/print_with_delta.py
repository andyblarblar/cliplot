# Script that prints all lines from a file, with a time delay

import sys
import time

if len(sys.argv) < 2:
    print("Usage: python3 print_with_delta.py <delta_seconds>")
    exit(-1)

delta_s = float(sys.argv[1])
while True:
    with open('./noisy_pedal.txt') as f:
        for line in f.readlines():
            print(f"${line.strip()}$")
            sys.stdout.flush()
            time.sleep(delta_s)

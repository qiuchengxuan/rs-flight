#!/usr/bin/env python3
import os
import sys

import crcmod

if len(sys.argv) <= 1:
    print('Usage: convert.py filename')
    sys.exit(0)

if not sys.argv[1].endswith('.mcm'):
    sys.exit(-1)

output = bytearray()
crc32 = crcmod.mkCrcFun(0x104c11db7, rev=False)
with open(sys.argv[1]) as mcm:
    if mcm.readline() != 'MAX7456\n':
        sys.exit(-1)
    for index in range(0, 256):
        for i in range(0, 64):
            number = int(mcm.readline(), 2)
            output.extend(number.to_bytes(1, byteorder='big'))
os.write(1, b'7456')
os.write(1, crc32(output).to_bytes(4, byteorder='big'))
os.write(1, output)

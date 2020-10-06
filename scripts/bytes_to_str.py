#!/usr/bin/env python3
import sys


def main():
    line = next(sys.stdin).strip()
    parts = line.split(' ', 1)
    if len(parts) < 2:
        return
    line = parts[1][1:-1]
    print(''.join(chr(int(b, 16)) for b in line.split(',')), )


if __name__ == '__main__':
    main()

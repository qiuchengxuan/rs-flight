#!/usr/bin/env python3
import sys
from pathlib import Path

from cli import CLI


def main():
    if len(sys.argv) < 2:
        print("Serial not specified")
        return

    path = sys.argv[1]
    if not Path(path).exists():
        print("Not found:", sys.argv[1])
        return

    cli = CLI(path)
    print(cli.tx('telemetry'))
    cli.close()

if __name__ == '__main__':
    main()

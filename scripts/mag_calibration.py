#!/usr/bin/env python3
import sys
import time
from pathlib import Path

import numpy
from cli import CLI
from telemetry import read_sensor


def main():
    if len(sys.argv) < 2:
        print("Serial not specified")
        return

    path = sys.argv[1]
    if not Path(path).exists():
        print("Not found:", sys.argv[1])
        return

    cli = CLI(path)

    try:
        min_value = numpy.array([sys.maxsize, sys.maxsize, sys.maxsize])
        max_value = -min_value
        for _ in range(30 * 50):
            data = read_sensor(cli, 'magnetism')
            for axis in range(3):
                min_value[axis] = min(min_value[axis], data[axis])
                max_value[axis] = max(max_value[axis], data[axis])
            print('min: %s, max: %s' % (str(min_value), str(max_value)), end='\r')
            time.sleep(0.02)
        offset = [(min_value[axis] + max_value[axis]) // 2 for axis in range(3)]
        print('')
        print('offset: ', offset)
    except EOFError:
        pass

    cli.close()


if __name__ == '__main__':
    main()
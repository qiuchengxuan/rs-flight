#!/usr/bin/env python3
import sys
from pathlib import Path
import time


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
        _min = [0, 0, 0]
        _max = [0, 0, 0]
        for _ in range(30 * 50):
            data = read_sensor(cli, 'magnetism')
            for axis in range(3):
                _min[axis] = min(_min[axis], data[axis])
                _max[axis] = max(_max[axis], data[axis])
            print('min: %s, max: %s' % (str(_min), str(_max)), end='\r')
            time.sleep(0.02)
        offset = [(_min[axis] + _max[axis]) / 2 for axis in range(3)]
        print('')
        print('offset: ', offset)
    except EOFError:
        pass

    cli.close()


if __name__ == '__main__':
    main()

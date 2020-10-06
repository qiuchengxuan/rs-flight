#!/usr/bin/env python3
import sys
import time
from enum import Enum
from pathlib import Path
from typing import Dict

import numpy

from cli import CLI
from telemetry import get_sensitive, read_sensor


class Attitude(Enum):
    FLAT = 0
    REVERSE = 1
    ROLL_LEFT = 2
    ROLL_RIGHT = 3
    PITCH_DOWN = 4
    PITCH_UP = 5

    def __str__(self) -> str:
        return {
            Attitude.FLAT: 'flat',
            Attitude.REVERSE: 'reverse',
            Attitude.ROLL_LEFT: 'roll-left',
            Attitude.ROLL_RIGHT: 'roll-right',
            Attitude.PITCH_DOWN: 'pitch-down',
            Attitude.PITCH_UP: 'pitch-up',
        }[self]


def get_average(cli: CLI) -> numpy.array:
    avg = read_sensor('acceleration')
    for i in range(50):
        time.sleep(0.02)
        avg = (avg + read_sensor('acceleration')) / 2
    return avg


def collect(cli: CLI) -> Dict[Attitude, int]:
    data = {}

    while len(data) < len(Attitude):
        _ = input("Press enter to perform once calibration: ")
        averaged = get_average(cli)
        vector = averaged / numpy.linalg.norm(averaged)
        attitude, value = None, 0
        if vector[2] < -0.8:
            attitude, value = Attitude.FLAT, averaged[2]
        elif vector[2] > 0.8:
            attitude, value = Attitude.REVERSE, averaged[2]
        elif vector[0] < -0.8:
            attitude, value = Attitude.ROLL_LEFT, averaged[0]
        elif vector[0] > 0.8:
            attitude, value = Attitude.ROLL_RIGHT, averaged[0]
        elif vector[1] < -0.8:
            attitude, value = Attitude.PITCH_DOWN, averaged[1]
        elif vector[1] > 0.8:
            attitude, value = Attitude.PITCH_UP, averaged[1]
        if attitude is None:
            print("Invalid averaged", averaged)
        if attitude not in data:
            print("Add %s as %s" % (str(attitude), value))
        else:
            print("Update %s from %s to %s" % (str(attitude), data[attitude], value))
        data[attitude] = value
    return data


def main():
    if len(sys.argv) < 2:
        print("Serial not specified")
        return

    path = sys.argv[1]
    if not Path(path).exists():
        print("Not found:", sys.argv[1])
        return

    cli = CLI(path)
    sensitive = get_sensitive(cli, 'acceleration')

    try:
        data = collect(cli)
        bias = [
            (data[Attitude.ROLL_LEFT] + data[Attitude.ROLL_RIGHT]) / 2,
            (data[Attitude.PITCH_DOWN] + data[Attitude.PITCH_UP]) / 2,
            (data[Attitude.FLAT] + data[Attitude.REVERSE]) / 2,
        ]
        gain = [
            sensitive * sensitive / (data[Attitude.ROLL_RIGHT] - bias[0]),
            sensitive * sensitive / (data[Attitude.PITCH_UP] - bias[1]),
            sensitive * sensitive / (data[Attitude.REVERSE] - bias[2]),
        ]
        print("bias:", bias)
        print("gain:", gain)
    except EOFError:
        pass

    cli.close()


if __name__ == '__main__':
    main()

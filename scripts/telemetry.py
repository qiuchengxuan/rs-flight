import json

import numpy
from jsonpath import jsonpath

from cli import CLI


def get_sensitive(cli: CLI, sensor: str) -> int:
    output = json.loads(cli.tx('telemetry'))
    return jsonpath(output, 'raw.%s.sensitive' % sensor)[0]


def read_sensor(cli: CLI, sensor: str) -> numpy.array:
    output = json.loads(cli.tx('telemetry'))
    axes = jsonpath(output, 'raw.%s.axes' % sensor)[0]
    return numpy.array([axes['x'], axes['y'], axes['z']])

#!/usr/bin/pyhton

# Measure system timing using dwf and a Digital Discovery Board.

from digital_discovery.digital_discovery import DigitalDiscovery
import numpy as np
import pandas as pd
import subprocess
from pathlib import Path
import pytest

default_fs = 100e6
default_repetitions = 100
default_chip = "STM32F411RE"

cargo_command = "flash"


class TestFixture:
    def __init__(self):
        self.fs = default_fs
        self.repetitions = default_repetitions
        self.chip = default_chip

        # Prepare measurement device
        self._device = DigitalDiscovery()
        self._device.open_device()

    def run_test_case(self, test_name, timeframe, release=True, trigger_on_output=True):
        # Set-up instrument
        self._device.prepare_triggered_latency(self.fs, timeframe, trigger_on_output)

        # Flash program
        print("Flash program: {} ({})".format(test_name, "release" if release else "debug"))
        res = subprocess.run([
            "cargo", cargo_command,
            "--chip={}".format(self.chip),
            "--",
            "--test={}".format(test_name),
            "--release" if release else "-q"],
            capture_output=True,
            cwd="../arm_cm4")
        if res.returncode != 0:
            raise Exception("Could not flash MCU: {}".format(res.stderr.decode()))

        # Measure time
        print("Measure latency", end="")
        time = np.zeros(self.repetitions)
        for i in range(0, self.repetitions):
            print(".", end="")
            self._device.measure()
            time[i] = self._device.evaluate_latency()
        print()

        # Store results
        Path("result").mkdir(exist_ok=True)
        df = pd.DataFrame()
        df[test_name] = time
        df.to_csv("result/{}_{}.csv".format(test_name, "release" if release else "debug"))

        return time

    def close(self):
        self._device.close_device()


@pytest.fixture
def fixture():
    fixture = TestFixture()
    yield fixture
    fixture.close()

#!/usr/bin/pyhton

# Measure interrupt latency using dwf and a Digital Discovery Board.

from digital_discovery.digital_discovery import DigitalDiscovery
import numpy as np
import pandas as pd
import subprocess
from pathlib import Path

cargo_command = "flash"
chip = "STM32F411RE"

# (cargo test name, "release"/"debug" build, acquisition time)
test_cases = [
    ("arm_cm4-latency-isr-bypass", "release", 100e-6),
    ("arm_cm4-latency-isr-kernel", "release", 100e-6),
    #("arm_cm4-latency-isr-kernel", "debug", 20e-3),
]

measurement_iterations = 50


def run_test_case(device, test_case, fs):
    test = test_case[0]
    build_type = test_case[1]
    t = test_case[2]

    # Set-up instrument
    device.prepare_triggered_latency(fs, t)

    # Flash program
    print("Flash program: {} ({})".format(test, build_type))
    subprocess.run([
        "cargo", cargo_command,
        "--chip={}".format(chip),
        "--",
        "--test={}".format(test),
        "--release" if build_type == "release" else ""],
        capture_output=True,
        cwd="../arm_cm4")

    # Measure latency
    print("Measure latency", end="")
    latency = np.zeros(measurement_iterations)
    for i in range(0, measurement_iterations):
        print(".", end="")
        device.measure()
        latency[i] = device.evaluate_latency()
    print()

    return latency


def main():
    fs = 100e6

    # Prepare measurement device
    device = DigitalDiscovery()
    device.open_device()

    # Run measurements
    latencies = pd.DataFrame()
    for test_case in test_cases:
        latency = run_test_case(device, test_case=test_case, fs=fs)
        latencies["{}_{}".format(test_case[0], test_case[1])] = latency

    device.close_device()

    # Store results
    Path("result").mkdir(exist_ok=True)
    latencies.to_csv("result/raw.csv")

    stats = latencies.describe()
    stats.to_csv("result/stats.csv")


if __name__ == "__main__":
    main()

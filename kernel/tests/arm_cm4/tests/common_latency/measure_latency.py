#!/usr/bin/pyhton

from pydwf import DwfLibrary, PyDwfError, DwfDigitalOutOutput, DwfDigitalOutIdle
from pydwf.utilities import openDwfDevice
import time


def measure(device):
    channel = 0
    # Get a reference to the device's DigitalOut instrument.
    digital_out = device.digitalOut

    # Use the DigitalOut instrument.
    digital_out.reset()

    digital_out.runSet(run_duration=0.1)
    digital_out.repeatSet(repeat=1)

    digital_out.idleSet(channel_index=channel, idle_mode=DwfDigitalOutIdle.High)
    digital_out.counterInitSet(channel_index=channel, high=True, counter_init=0)
    digital_out.counterSet(channel_index=channel, low_count=1, high_count=2)
    digital_out.enableSet(channel_index=channel, enable_flag=True)

    digital_out.configure(start=True)

    time.sleep(1)


def main():
    try:
        dwf = DwfLibrary()
        with openDwfDevice(dwf) as device:
            print("Run measurement")
            measure(device)
    except PyDwfError as exception:
        print("PyDwfError:", exception)
    except KeyboardInterrupt:
        print("Keyboard interrupt, ending demo.")


if __name__ == "__main__":
    main()

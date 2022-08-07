#!/usr/bin/pyhton

# Measure interrupt latency using dwf and a Digital Discovery Board.

from ctypes import *
from .dwfconstants import *
import time
import sys
import numpy
from numba import njit


class DigitalDiscovery:
    dwf = None
    hdwf = c_int()
    fs = None
    t = None
    n_samples = None
    samples = None

    def __init__(self):
        if sys.platform.startswith("win"):
            self.dwf = cdll.dwf
        elif sys.platform.startswith("darwin"):
            self.dwf = cdll.LoadLibrary("/Library/Frameworks/dwf.framework/dwf")
        else:
            self.dwf = cdll.LoadLibrary("libdwf.so")

    def open_device(self):
        version = create_string_buffer(6)
        self.dwf.FDwfGetVersion(version)
        print("DWF Version: " + str(version.value))

        print("Opening first device")
        self.dwf.FDwfDeviceOpen(c_int(-1), byref(self.hdwf))

        if self.hdwf.value == 0:
            print("failed to open device")
            szerr = create_string_buffer(512)
            self.dwf.FDwfGetLastErrorMsg(szerr)
            print(str(szerr.value))
            quit()

    def close_device(self):
        self.dwf.FDwfDeviceCloseAll()

    """ Prepare the device for a latency measurement triggered by an digital output.
    I/Os:
        * trigger output: DIO24 (pulse 10ms low, 90ms high, idle: high)
        * trigger input: DIN1 (connect to DIO24) if trigger_on_output else DIN0
        * latency capture input: DIN0
    """
    def prepare_triggered_latency(self, fs, t, trigger_on_output):
        self.fs = fs
        self.t = t

        # Prepare a pulse (idle: high, low for 10ms, high for 90ms on DIO24
        self.dwf.FDwfDigitalOutEnableSet(self.hdwf, c_int(0), c_int(1))
        self.dwf.FDwfDigitalOutDividerSet(self.hdwf, c_int(0), c_int(1000))
        self.dwf.FDwfDigitalOutCounterSet(self.hdwf, c_int(0), c_int(1000), c_int(9000))
        self.dwf.FDwfDigitalOutRunSet(self.hdwf, c_double(0.1))
        self.dwf.FDwfDigitalOutIdleSet(self.hdwf, c_int(2))  # idle high
        self.dwf.FDwfDigitalOutCounterInitSet(self.hdwf, c_int(0), c_int(1), c_int(0))

        # Prepare input capture on DIN0..7
        hz_di = c_double()
        self.dwf.FDwfDigitalInInternalClockInfo(self.hdwf, byref(hz_di))
        self.dwf.FDwfDigitalInDividerSet(self.hdwf, c_int(int(hz_di.value / fs)))
        self.dwf.FDwfDigitalInSampleFormatSet(self.hdwf, c_int(8))
        self.n_samples = int(t * fs)
        self.dwf.FDwfDigitalInBufferSizeSet(self.hdwf, c_int(self.n_samples))

        # Set Trigger to falling edge of DIN1 or DIN0
        self.dwf.FDwfDigitalInTriggerSourceSet(self.hdwf, c_ubyte(3))  # trigsrcDetectorDigitalIn
        self.dwf.FDwfDigitalInTriggerPositionSet(self.hdwf, c_int(int(self.n_samples)))
        if trigger_on_output:
            self.dwf.FDwfDigitalInTriggerSet(self.hdwf, c_int(0), c_int(0), c_int(0), c_int(2))  # DI1 falling edge
        else:
            self.dwf.FDwfDigitalInTriggerSet(self.hdwf, c_int(0), c_int(0), c_int(0), c_int(1))  # DIO falling edge

    def measure(self):
        sts = c_byte()
        # Begin acquisition
        self.dwf.FDwfDigitalInConfigure(self.hdwf, c_bool(False), c_bool(True))
        # Active output
        self.dwf.FDwfDigitalOutConfigure(self.hdwf, c_int(1))
        while True:
            self.dwf.FDwfDigitalInStatus(self.hdwf, c_int(1), byref(sts))
            if sts.value == stsDone.value:
                break
            time.sleep(0.01)

        # Get samples
        samples = (c_uint8 * self.n_samples)()
        self.dwf.FDwfDigitalInStatusData(self.hdwf, samples, self.n_samples)

        np_samples = numpy.fromiter(samples, dtype=numpy.uint8)
        self.samples = np_samples
        return np_samples

    """ Evaluate latency from trigger.
    Assumptions:
        * trigger set to sample index 0
        * latency capture channel is DIN0
    """
    def evaluate_latency(self):
        return self._evaluate_latency_din0(self.samples, self.fs)

    @staticmethod
    @njit
    def _evaluate_latency_din0(samples, fs):
        for i, sample in enumerate(samples):
            if (sample & 0x01) == 0x01:
                return i / fs

        return -1

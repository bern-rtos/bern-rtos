from test_fixture import fixture
import numpy as np

release=True

def test_isr_bypass(fixture):
    timing = fixture.run_test_case(test_name="arm_cm4-timing-isr-bypass", timeframe=1e-3, release=release)
    assert np.max(timing) < 100e-6
    assert np.min(timing) > 0


def test_isr_kernel(fixture):
    timing = fixture.run_test_case(test_name="arm_cm4-timing-isr-kernel", timeframe=1e-3, release=release)
    assert np.max(timing) < 100e-6
    assert np.min(timing) > 0


def test_semaphores(fixture):
    timing = fixture.run_test_case(test_name="arm_cm4-timing-semaphore", timeframe=1e-3, trigger_on_output=False, release=release)
    assert np.max(timing) < 200e-6
    assert np.min(timing) > 0

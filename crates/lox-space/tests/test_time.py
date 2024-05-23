import lox_space as lox
import pathlib
import pytest

DATA_DIR = pathlib.Path(__file__).parents[3].joinpath('data')

@pytest.fixture
def provider():
    return lox.UT1Provider(str(DATA_DIR.joinpath('finals2000A.all.csv')))

def test_utc(provider):
    utc_exp = lox.UTC(2000, 1, 1)
    utc_act = utc_exp.to_tai().to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_tcb().to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_tcg().to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_tdb().to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_tt().to_utc()
    assert utc_exp == utc_act
    utc_act = utc_exp.to_ut1(provider).to_utc(provider)
    assert utc_exp == utc_act

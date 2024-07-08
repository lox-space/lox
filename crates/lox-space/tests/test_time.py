import lox_space as lox
import pytest


def test_time(provider):
    tai_exp = lox.Time("TAI", 2000, 1, 1)
    tai_act = lox.Time.from_iso("2000-01-01T00:00:00.000 TAI")
    assert tai_exp == tai_act
    tai_act = tai_exp.to_tai()
    assert tai_exp == tai_act
    tai_act = tai_exp.to_tcb().to_tai()
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_tcg().to_tai()
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_tdb().to_tai()
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_tt().to_tai()
    assert tai_exp.isclose(tai_act)
    tai_act = tai_exp.to_ut1(provider).to_tai(provider)
    assert tai_exp.isclose(tai_act)
    with pytest.raises(ValueError):
        tai_exp.to_ut1()
    tai1 = lox.Time("TAI", 2000, 1, 1, 0, 0, 0.5)
    assert tai1 > tai_exp
    assert tai1 >= tai_exp
    assert tai_exp < tai1
    assert tai_exp <= tai1
    assert tai_exp != tai1
    dt = lox.TimeDelta(0.5)
    assert tai_exp + dt == tai1
    assert tai1 - dt == tai_exp
    assert tai1 - tai_exp == dt


def test_utc(provider):
    utc_exp = lox.UTC(2000, 1, 1)
    utc_act = lox.UTC.from_iso("2000-01-01T00:00:00.000")
    assert utc_exp == utc_act
    utc_act = lox.UTC.from_iso("2000-01-01T00:00:00.000Z")
    assert utc_exp == utc_act
    utc_act = lox.UTC.from_iso("2000-01-01T00:00:00.000 UTC")
    assert utc_exp == utc_act
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


def test_time_delta():
    delta = lox.TimeDelta(1.5)
    assert str(delta) == "1.5 seconds"
    assert delta.seconds() == 1
    assert delta.subsecond() == 0.5
    assert str(delta + delta) == "3 seconds"
    assert str(delta - delta) == "0 seconds"
    assert str(-delta) == "-1.5 seconds"
    with pytest.raises(ValueError):
        lox.TimeDelta(float("nan"))

use std::iter::zip;

use lox_time::{base_time::BaseTime, deltas::TimeDelta};
use lox_utils::roots::{Brent, FindBracketedRoot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Crossing {
    Up,
    Down,
}

impl Crossing {
    fn new(s0: f64, s1: f64) -> Option<Crossing> {
        if s0 < 0.0 && s1 > 0.0 {
            Some(Crossing::Up)
        } else if s0 > 0.0 && s1 < 0.0 {
            Some(Crossing::Down)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub crossing: Crossing,
    pub time: BaseTime,
}

pub fn find_events<F: Fn(f64) -> f64>(f: F, time0: BaseTime, deltas: &[TimeDelta]) -> Vec<Event> {
    let mut events = vec![];

    let root_finder = Brent::default();

    let n = deltas.len();
    if n < 2 {
        return events;
    }
    let time1 = time0 + *deltas.last().unwrap();

    let signs: Vec<f64> = deltas
        .iter()
        .map(|&delta| f(delta.to_decimal_seconds()).signum())
        .collect();

    if signs.iter().all(|&s| s > 0.0) {
        return events;
    }

    if signs.iter().all(|&s| s < 0.0) {
        return vec![
            Event {
                crossing: Crossing::Up,
                time: time0,
            },
            Event {
                crossing: Crossing::Down,
                time: time1,
            },
        ];
    }

    let ts0 = zip(&deltas[0..n], &signs[0..n]);
    let ts1 = zip(&deltas[1..], &signs[1..]);
    zip(ts0, ts1).for_each(|((&delta0, &s0), (&delta1, &s1))| {
        if let Some(crossing) = Crossing::new(s0, s1) {
            let t = root_finder
                .find_root(
                    &f,
                    (delta0.to_decimal_seconds(), delta1.to_decimal_seconds()),
                )
                .expect("sign changed but root finder failed");
            let time = time0 + TimeDelta::from_decimal_seconds(t).unwrap();

            events.push(Event { crossing, time });
        }
    });

    if events.first().unwrap().crossing == Crossing::Down {
        events.insert(
            0,
            Event {
                crossing: Crossing::Up,
                time: time0,
            },
        );
    }

    if events.last().unwrap().crossing == Crossing::Up {
        events.push(Event {
            crossing: Crossing::Down,
            time: time1,
        });
    }

    events
}

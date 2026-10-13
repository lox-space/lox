<!--
SPDX-FileCopyrightText: 2026 Eivind Bergem <eivind.bergem@gmail.com>

SPDX-License-Identifier: MPL-2.0
-->

# Analysis pipeline

## Pipeline

The `Pipeline` trait is the foundational buidling block. It has two
required methods, and a lot of provided methods.

A pipeline works over a collection of assets – that can be a ground
station/spacecraft pair, link, or something else – and collection of
windows for each asset.

The pipeline is lazy evaluated, so nothing happens until `compute()`
is called. The basic pipeline computes sequentially. There are
separate traits for parallel and stream processing –
`ParallelPipeline` and `StreamPipeline` – that have blanket
implementation for any `T: Pipeline`.

`AssetPairAnalysis` can be used to start a pipeline:

```Rust
let pipeline = AssetPairanalysis::new(scenario.ground_stations(), scenario.spacecraft(), initial_interval);
```


### Refine

`refine()` is used to refine a windows for each asset. The closure
should take an asset and a window as arguments and return an iterator
over new windows.

```Rust
pipeline.refine(|(gs, sc), interval| {
    // refine pipeline
})
```

### Filter

`filter()` is used to filter out assets and windows. It takes a
closure that takes an asset and a window as arguments and returns
bool, dropping the window if it returns false. Assets with no more
windows are dropped.

```Rust
pipeline.filter(|(gs, sc), interval| {
    gd.id() != AssetId("Svalbard") || interval > TimeInterval::from_secs(60)
})
```

There's also `filter_asset()` and `filter_window()`.

### Map asset

`map_asset()` maps an asset into an iterator over new asset types,
f.ex. to map a GS-SC pair to terminal pairs or links.

## Composability

The pipeline works over _assets_ and _windows_. Many of the
transformations are done independently on either asset or
windows. Using the traits `Asset` and `Window` (names can be bikeshed
later), we can do transformations on assets and windows. These
building blocks can then be used to create a pipeline, f.ex. a
complete link budget pipeline:

```Rust
AssetPairAnalysis::new(&self.gs, &self.sc, self.interval)
    .refine(move |(gs, sc), interval| {
        let sc_traj = self.ensemble.get(sc.id()).unwrap();

        interval.elevation_detect(UniformSampler::new(step), gs, sc_traj)
    })
    .filter_window(|window| window.duration() > TimeDelta::from_seconds(40000))
    .map_asset(move |asset| asset.links(&self.ensemble, plan, overrides))
    .refine(move |link, interval| {
        let windows = interval.link_margin_detect(step, link)?.collect::<Vec<_>>();

        Ok(windows.into_iter())
    })
```

## Computation

Pipelines can be executed using `compute()`, `par_compute()` or
`stream_compute()`. Execution is parallelized in `par_compute()` and
`stream_compute()`, running each intitial asset in a separate
thread/task. In the link budget example showed in the previous
section, this means that each link is not processed in parallel, only
each GS-SC pair.

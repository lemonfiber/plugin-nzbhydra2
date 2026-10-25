# NZBHydra2 for lemonfiber

**A search box across all your Usenet indexers at once.**
NZBHydra2 is one of the parts the stack lemonfiber ships is built from.
Without it: Search one indexer at a time.

## How lemonfiber carries it

The default bundle in
[lemonfiber-plugins](https://github.com/lemonfiber/lemonfiber-plugins) pins this
repository by a revision and the digests of its manifest and its tree, and
lemonfiber compiles that tree in. lemonfiber installs NZBHydra2 with the adapter
this repository holds, which speaks lemonfiber's `owner.claim` and
`indexer.search` contracts in front of NZBHydra2: a Rust crate built on the
core's adapter kit, into the image `ghcr.io/lemonfiber/nzbhydra2-adapter`,
published from every commit on `main` and from each tag the release train cuts.
It claims a fresh NZBHydra2 by turning authentication on for the administrator
lemonfiber mints, and reads which indexers it searches and how hard each was
used. lemonfiber reaches NZBHydra2 through this adapter alone. lemonfiber mints
the administrator's name and password and writes them into the adapter's
`upstream.json`, and the adapter's claim makes NZBHydra2 ask for them.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The part's manifest |
| `recordings/nzbhydra2/` | The responses its proofs are held to |
| `src/` | The adapter |
| `conformance/owner.claim@1/` | The adapter's answer to each conformance case, in front of the pinned NZBHydra2 |
| `conformance/indexer.search@1/` | The adapter's answer to each conformance case, in front of the pinned NZBHydra2 |
| `Dockerfile` | The adapter's image |
| `targets.toml` | The lemonfiber its checks run against, and the core the adapter is built on |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
proof against its recording, and every pinned digest. The adapter is held to its
format, its lints, its tests, full line coverage and its dependency policy, its
image runs as lemonfiber runs it, and `recordings` takes every recording again
against the pinned NZBHydra2. `just ci` runs the gates that need nothing but
this tree.

## Licence

The code and data in this repository are under the licence in `LICENSE`.
NZBHydra2 itself is under Apache-2.0 and is not distributed here: this
repository names an image, it does not contain one.

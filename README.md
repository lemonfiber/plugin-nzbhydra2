# NZBHydra2 for lemonfiber

**A search box across all your Usenet indexers at once.**
NZBHydra2 is one of the parts the stack lemonfiber ships is built from.
Without it: Search one indexer at a time.

## How lemonfiber carries it

The default bundle in
[lemonfiber-plugins](https://github.com/lemonfiber/lemonfiber-plugins) pins this
repository by a revision and the digests of its manifest and its tree, and
lemonfiber compiles that tree in.
This repository holds the adapter that speaks lemonfiber's `owner.claim` and
`indexer.search` contracts in front of NZBHydra2: a Rust crate built on the core's
adapter kit, into the image `ghcr.io/lemonfiber/nzbhydra2-adapter`, published from
every commit on `main` and from each tag the release train cuts. It claims a
fresh NZBHydra2 by turning authentication on for the administrator lemonfiber
mints, and reads which indexers it searches and how hard each was used.
`plugin.toml` does not name the adapter yet, so lemonfiber assembles NZBHydra2
into the stack it ships as a stack service, and reaches it through the
in-process adapter its `api` names.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The part's manifest |
| `recordings/nzbhydra2/` | The responses its claims are proved against |
| `src/` | The adapter |
| `Dockerfile` | The adapter's image |
| `targets.toml` | The lemonfiber its checks run against, and the core the adapter is built on |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
claim against its recording, and every pinned digest. The adapter is held to its
format, its lints, its tests, full line coverage and its dependency policy, its
image runs as lemonfiber runs it, and `recordings` takes every recording again
against the pinned NZBHydra2. `just ci` runs the gates that need nothing but
this tree.

## Licence

The code and data in this repository are under the licence in `LICENSE`.
NZBHydra2 itself is under Apache-2.0 and is not distributed here: this
repository names an image, it does not contain one.

# NZBHydra2 for lemonfiber

**A search box across all your Usenet indexers at once.**
NZBHydra2 is one of the parts the stack lemonfiber ships is built from.
Without it: Search one indexer at a time.

## How lemonfiber carries it

The default bundle in
[lemonfiber-plugins](https://github.com/lemonfiber/lemonfiber-plugins) pins this
repository by a revision and the digests of its manifest and its tree, and
lemonfiber compiles that tree in.
Until its adapter ships, lemonfiber assembles NZBHydra2 into the stack it ships as a
stack service, and reaches it through the in-process adapter its `api` names.
Its image is NZBHydra2's own, so the release train never tags this repository, and
its pin moves by a reviewed pull request on the catalogue.

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The part's manifest |
| `recordings/nzbhydra2/` | The responses its claims are proved against |
| `targets.toml` | The lemonfiber its checks run against |
| `proofs.json` | What those checks said, committed |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
claim against its recording, and every pinned digest. This repository holds no
Python of its own. `just ci` runs the gates that need nothing but this tree.

## Licence

The data in this repository is under the licence in `LICENSE`.
NZBHydra2 itself is under Apache-2.0 and is not distributed here: this
repository names an image, it does not contain one.

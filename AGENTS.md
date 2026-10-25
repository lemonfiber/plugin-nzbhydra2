# AGENTS.md — plugin-nzbhydra2

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

NZBHydra2 as a first-party lemonfiber plugin: `plugin.toml` and the adapter that
speaks the `owner.claim@1` and `indexer.search@1` contracts in front of NZBHydra2.
The adapter is the Rust crate at the root, built on `lemonfiber-adapter-kit` from
the core at the commit `Cargo.toml` pins, into the image the `Dockerfile` builds.
`release.yml` publishes that image from every commit on `main` and from the
train's tag. The
[bundle contract](https://github.com/lemonfiber/spec/blob/main/20-architecture/contracts/bundle.md)
says how the catalogue pins this repository and how lemonfiber carries it.
`plugin.toml` runs NZBHydra2 beside the adapter, `nzbhydra2-adapter`, which `fronts`
it, and the conformance recordings under `conformance/` are the adapter's answers.

## The rules you cannot break

- **The plugin id is the service's id** (`ARCH-R266`). The container, the
  configuration directory, the settings and the journal keep their names.
- **Its proofs' recordings sit under `recordings/nzbhydra2/`**, and its starting
  files, where it has any, under `config/nzbhydra2/` (`ARCH-R271`).
- **Every service declares `reaches` and `asks_for`** (`ARCH-R270`).
- **The reader is spec's `first-party-plugin` workflow's,** and lemonfiber
  records the recordings itself. There is no Python here.
- **No `unwrap`, `expect`, `panic` or indexing** outside tests. The lints in
  `Cargo.toml` are the core's.
- **The adapter never writes its credential.** lemonfiber mints the
  administrator's name and password and writes them into the adapter's
  `upstream.json` as `username` and `password` (`ARCH-R315`, `ARCH-R317`); the
  adapter reads them for every call and never logs them.
- **A claim changes nothing NZBHydra2 already holds** (`ARCH-R328`): only an
  NZBHydra2 with no authentication is guarded, and one guarded for anybody else
  is answered as theirs.
- **What NZBHydra2 cannot do is answered `unsupported`** (`ARCH-R334`): it is
  told about no application and reads from no aggregator.
- **The core's crates move with the train.** The `rev` in `Cargo.toml` is the
  core commit `targets.toml` names, the same for every `lemonfiber-*` crate.
- **Moving NZBHydra2's digest** means recording every response again against the
  new image in the same change, moving `RELEASE` and `DIGEST` in
  `src/adapter.rs` with it, and committing the `proofs.json` the checks write.

## Checks

```
just ci
```

The gates that need nothing but this tree. The plugin verdict and the
recordings run in CI; `just recordings` takes the recordings again locally, with
Docker.

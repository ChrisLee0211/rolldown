# Rolldown

![CI status](https://github.com/rolldown-rs/rolldown/actions/workflows/ci-correctness.yaml/badge.svg) [![MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast JavaScript/TypeScript bundler in Rust with Rollup-compatible API.

## Status

Currently we are targeting to pass the [function tests](https://github.com/rollup/rollup/tree/master/test/function) of Rollup.

```
┌────────────┬────────┐
│  (index)   │ Values │
├────────────┼────────┤
│   total    │  869   │
│   failed   │   0    │
│ skipFailed │  364   │
│  ignored   │   8    │
│  skipped   │   0    │
│   passed   │  497   │
└────────────┴────────┘
```

## Authors

- Yinan Long ([Brooooooklyn](https://github.com/Brooooooklyn))
- Yunfei He ([hyf0](https://github.com/hyf0))
- [h-a-n-a](https://github.com/h-a-n-a)
- [IWANABETHATGUY](https://github.com/IWANABETHATGUY)

# Credits

The Rolldown project are standing upon the shoulders of these giants:

- [Rollup](https://github.com/rollup/rollup), created by [Rich-Harris](https://github.com/Rich-Harris) and maintained by [lukastaegert](https://github.com/lukastaegert).
- [esbuild](https://github.com/evanw/esbuild), created by [evanw](https://github.com/evanw).

# THIRD-PARTY-LICENSE

This project partially copies code from the following projects:

- [Rollup(MIT)](https://github.com/rollup/rollup/blob/680912e2ceb42c8d5e571e01c6ece0e4889aecbb/LICENSE-CORE.md)
- [esbuild(MIT)](https://github.com/evanw/esbuild/blob/0c8a0a901d9a6c7bbff9b4dd347c8a3f65f6c6dd/LICENSE.md)

Licenses are list in [THIRD-PARTY-LICENSE](/THIRD-PARTY-LICENSE)


# Contributing to zerOS

## General notes

As far as you're kind to contributors / other users and you respect the [code of conduct](CODE_OF_CONDUCT.md), you're very welcome to join this long development journey.
We accept any contributions, from bug fixes to complete new features (as far as it is interesting and not _too_ crazy).

All new contributions should be made by github pull requests.

## zerOS kernel

### Coding guidelines

#### Style

So far, no particular style is _really_ enforced. However, it would be very appreciated, as the codebase will grow, to stick to the `.clang-format` we provide.
Any request to modify the style must be accompanied by a valid rationale.

#### APIs

Names of functions from kernel subsystems APIs should be prefixed with `zerOS_`. You're otherwise free to name other `static` functions as you see fit.

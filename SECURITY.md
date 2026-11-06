# Security policy

## Reporting a vulnerability

Report it privately, through GitHub's private vulnerability reporting:
[report a vulnerability](https://github.com/Vehicoule/nsh-m3e/security/advisories/new). Do not open a public issue,
pull request or discussion about it.

Say what is affected (the crate and the commit), how to reproduce it, and what an attacker gains.

## Supported versions

`nsh-m3e` has no release yet. Only the latest commit of `main` is supported.

## Scope

In scope:

- **The library**: input from an app that makes a component panic, loop forever or grow memory without bound.
- **The supply chain of this repository**: its CI workflows, including the ones that open updates of NSH and of the
  androidx tokens and the one that merges NSH updates on its own (ADR 0014; androidx updates are merged by a person);
  its pinned dependencies; its vendored sources and the tokens generated from them.

Out of scope: vulnerabilities in NSH itself; report those to [NSH](https://github.com/longbridge/nsh).

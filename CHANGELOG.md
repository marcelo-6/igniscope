<!-- markdownlint-disable -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-11

### <!-- 0 -->Features
- Add deterministic parse of project resouces and basic analysis  by @marcelo-6 ([6d45e66](https://github.com/marcelo-6/igniscope/commit/6d45e661be96951b12afb51399b20b182f6dc46b))

- Classify project resources and compute per-project coverage metrics (known and unknown resources)  by @marcelo-6 ([f17f7bb](https://github.com/marcelo-6/igniscope/commit/f17f7bb19b15cf384e73ddabc1246fc894491610))

- Add project resource discovery validation  by @marcelo-6 ([7ffe5bb](https://github.com/marcelo-6/igniscope/commit/7ffe5bbb2cf4e9e8e6bae109581f0e530e834386))

- Parse project.json metadata  by @marcelo-6 ([7563706](https://github.com/marcelo-6/igniscope/commit/75637068585f4f576baf011b09d5e0dcc4b331cd))

- Detect archive kind and project roots for project exports and gateway backups  by @marcelo-6 ([9dfe240](https://github.com/marcelo-6/igniscope/commit/9dfe2401a6444040ece37e40f23b56c1e9d79955))

- Scaffold Clap CLI parsing and added basic tests  by @marcelo-6 ([d9c2a14](https://github.com/marcelo-6/igniscope/commit/d9c2a14b3a7d6d489bbe5d1e6b7fa229f07d1c76))


### <!-- 1 -->Bug Fixes
- Fix assertion for just release  ([dad0b5b](https://github.com/marcelo-6/igniscope/commit/dad0b5bc873b1d06b96f21d8a26591bfc7ba0238))


### <!-- 3 -->Documentation
- Add codecov report badge to readme  by @marcelo-6 ([68021d2](https://github.com/marcelo-6/igniscope/commit/68021d22df07c638dee30af11b112514a575b1eb))

- Add metadata about project to cargo.toml  by @marcelo-6 ([128c178](https://github.com/marcelo-6/igniscope/commit/128c178c6e2a10dcfd32382021a22873e290228a))

- Add readme  by @marcelo-6 ([49b1415](https://github.com/marcelo-6/igniscope/commit/49b141538c05fc8addf1c5a1674c2a7bba9b07ee))


### <!-- 5 -->Styling
- Cargo fmt  by @marcelo-6 ([9e9daa1](https://github.com/marcelo-6/igniscope/commit/9e9daa119dce235ad0036ebb1bb45f9dbfefc67c))


### <!-- 6 -->Testing
- Add test project/gateway backup files from Ignition exchange  by @marcelo-6 ([39aa4cd](https://github.com/marcelo-6/igniscope/commit/39aa4cd2380bfa2ef5bc2603023e8b11a5bb3f0d))

- Ignore test .zip and .gwbk files  by @marcelo-6 ([4bca5fc](https://github.com/marcelo-6/igniscope/commit/4bca5fc1bc68a026b3011b3c14cdca245aefa831))


### <!-- 7 -->Miscellaneous Tasks
- Ignore vscode workspace settings  ([cef96c1](https://github.com/marcelo-6/igniscope/commit/cef96c1c96623a478d7f8f125c530c1f1df6946c))

- Prepare v0.1.0  by @marcelo-6 in [#2](https://github.com/marcelo-6/igniscope/pull/2) ([1cdb320](https://github.com/marcelo-6/igniscope/commit/1cdb32091bda80c87764e3ab0f0f6945beb2278f))

- Preparing for cd workflow and release  by @marcelo-6 ([02920ba](https://github.com/marcelo-6/igniscope/commit/02920ba3c0ed89d6e0d1b328a1ceda1bed7e7a83))

- Add cd workflow to publish and create github release  by @marcelo-6 ([846dfaf](https://github.com/marcelo-6/igniscope/commit/846dfafa5733a385e11b79188418fbd04a26596c))

- Added color to release plan echo  by @marcelo-6 ([24a6829](https://github.com/marcelo-6/igniscope/commit/24a6829d73d7da051432777bd2849fc715cb7701))

- Ignore release notes  by @marcelo-6 ([9b6d4b0](https://github.com/marcelo-6/igniscope/commit/9b6d4b0c608c66f9d377d7c1b8cd0c9d227c4f09))

- Add release workflow to justfile  by @marcelo-6 ([147065e](https://github.com/marcelo-6/igniscope/commit/147065ef8852ff550b94b9cfd5baf1178a12ee69))

- Add dependabot version updates  by @marcelo-6 ([6a4e4a6](https://github.com/marcelo-6/igniscope/commit/6a4e4a66b610a57869619c34bc2828707a1fa0a1))

- Ci workflow testing complete  by @marcelo-6 ([c03afa6](https://github.com/marcelo-6/igniscope/commit/c03afa66d9296b3607790e050797cb66d14e57c7))

- Removed git-cliff hard requirement so ci can run just ci  by @marcelo-6 ([9bd5c4d](https://github.com/marcelo-6/igniscope/commit/9bd5c4de6f9efdb6eba82c24bb54dee0654a1ddf))

- Add ci workflow  by @marcelo-6 ([5742e66](https://github.com/marcelo-6/igniscope/commit/5742e6638b49419d2b83f3450f1f9a9ece4c3855))

- Fix license in cargo.toml  by @marcelo-6 ([0c5a446](https://github.com/marcelo-6/igniscope/commit/0c5a44665950802879a832f0f0b9ab79aba8f623))

- Add cargo publish to justfile  by @marcelo-6 ([3c4f1fb](https://github.com/marcelo-6/igniscope/commit/3c4f1fb9b77e06f426b6634dacb77d6852537e58))

- Add gitcliff config for changelog generation  by @marcelo-6 ([9daeeeb](https://github.com/marcelo-6/igniscope/commit/9daeeeb66b615e75c2187b196697108c9ae2386f))

- Add license  by @marcelo-6 ([a6c37e6](https://github.com/marcelo-6/igniscope/commit/a6c37e6ff41287b7b7001c716b1648a1228e5bb4))

- Added ci and converage commands to justfile  by @marcelo-6 ([553d369](https://github.com/marcelo-6/igniscope/commit/553d369093cf255139d6d87218265f334ceed242))

- Some todos for later  by @marcelo-6 ([d3cfc69](https://github.com/marcelo-6/igniscope/commit/d3cfc69814f5b8da0090380a08131587147c7e55))

- Add justfile for cargo commands  by @marcelo-6 ([9c7bcd3](https://github.com/marcelo-6/igniscope/commit/9c7bcd36062cfc8c282a173f9410c3107c9a39d4))

- Initial Commit  by @marcelo-6 ([4505aab](https://github.com/marcelo-6/igniscope/commit/4505aaba8d9c6d1f240d4ee812bcc7ff2f849859))


### <!-- 9 -->Revert
- Track release notes to allow manual changes before being released  by @marcelo-6 ([2c96469](https://github.com/marcelo-6/igniscope/commit/2c96469d83bed1b82bc85c59abf48f13c996e176))


### New Contributors
* @marcelo-6 made their first contribution in [#1](https://github.com/marcelo-6/igniscope/pull/1)


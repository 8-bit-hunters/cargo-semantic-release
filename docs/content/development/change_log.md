---
title: Change Log
cascade:
    type: docs
---

# V1.0.0

## Major

- 💥 Remove error handling from Changes constructor ([3c8e19b](https://github.com/8-bit-hunters/cargo-semantic-release/commit/3c8e19bf7fe7ba3dd6f0489374a5200fad124377))
- 💥 Implement PartialEq trait instead of old method ([8e50d69](https://github.com/8-bit-hunters/cargo-semantic-release/commit/8e50d697e09574e94ea0af1136f3ef73ca3519f0))
- 💥 Define better name for SemanticVersionAction ([6526d45](https://github.com/8-bit-hunters/cargo-semantic-release/commit/6526d452b96c2f6c2fef067dcbe230f9a3927552))
- 💥 Move get commits to Changes struct ([1aafcd8](https://github.com/8-bit-hunters/cargo-semantic-release/commit/1aafcd8ff1dce677829166bcd6aadbb2eaec86da))
- 💥 Move semantic version action evaluation to Changes struct ([22f2916](https://github.com/8-bit-hunters/cargo-semantic-release/commit/22f2916b86f1a4cb02b8ccebd15db1eb059f907a))

## Minor

- ✨ Implement TryFrom for Changes struct ([20e9bad](https://github.com/8-bit-hunters/cargo-semantic-release/commit/20e9bad8015cc2f1c5abbd73f4c03a4ecc97127f))
- ✨ Show version of the package when --version option is provided. ([de57a8b](https://github.com/8-bit-hunters/cargo-semantic-release/commit/de57a8b465487e66d18806d1bf4aefcd1d9aac1c))
- ✨ Fetch the commits until the last version tag ([fba240c](https://github.com/8-bit-hunters/cargo-semantic-release/commit/fba240cde4fca9550d1550e60b9bd455c60f1eae))
- ➕ Add Regex as dependency to the project ([1f538f0](https://github.com/8-bit-hunters/cargo-semantic-release/commit/1f538f01a8d5f8c2c1db76b535db78bb86350b22))
- ➕ Add semver as dependency ([b737339](https://github.com/8-bit-hunters/cargo-semantic-release/commit/b737339873c1ef14764e78997a5073ae879eabd2))

## Patch

- 🥅 Do error printouts instead of panic in the program ([2973be6](https://github.com/8-bit-hunters/cargo-semantic-release/commit/2973be6b95744b1d868fce87b97e4f54463f371e))
- 🚨 Fix empty line and trailing whitespaces. ([36e24ed](https://github.com/8-bit-hunters/cargo-semantic-release/commit/36e24ed427cba534288baa6aa9d7f47c457cd859))
- 🚨 Remove needless borrow (clippy). ([48730d7](https://github.com/8-bit-hunters/cargo-semantic-release/commit/48730d7d0c8adfbc42e0aa911158c4c3b139c425))
- 🚨 Remove unnecessary fallible conversion (clippy). ([d546a87](https://github.com/8-bit-hunters/cargo-semantic-release/commit/d546a87be1db589de8635abb5c6a14420b352b12))
- 🔧 Add Jan Willems as author to the project ([0d4d497](https://github.com/8-bit-hunters/cargo-semantic-release/commit/0d4d497c77d66dc2f6517549e773016eae89fc3e))
- ♻️ Move tag checking responsibility to Changes structure ([c69b370](https://github.com/8-bit-hunters/cargo-semantic-release/commit/c69b3705f0ce9e310058f469af85b4d23648a3bb))
- 🏗️ Create repository module with RepositoryExtension crate ([735b7bb](https://github.com/8-bit-hunters/cargo-semantic-release/commit/735b7bbd877673c48d5cb4cfb0683107ea9c8892))
- ♻️ Use intention instead of tags to avoid confusion ([d87b5d6](https://github.com/8-bit-hunters/cargo-semantic-release/commit/d87b5d6c9e235e730e882bb05d303d2559ae1668))
- ✅ Add end-to-end integration tests ([46e644e](https://github.com/8-bit-hunters/cargo-semantic-release/commit/46e644e4e64877cc38dea841aac9733d15274f7f))
- 🔧 Configure test_util module to be public during testing ([d03b252](https://github.com/8-bit-hunters/cargo-semantic-release/commit/d03b2525092c1db5c9b22e2663529bbf1c3a4b51))
- ♻️ Do mocking when unit testing Changes module ([fd2381f](https://github.com/8-bit-hunters/cargo-semantic-release/commit/fd2381fcafa91ac0506d3f6b591ec6202b2e3c1a))
- 🧑‍💻 Create mock error for tests ([5d89e4a](https://github.com/8-bit-hunters/cargo-semantic-release/commit/5d89e4aae0b8649c5df6e19430c9261c550b72c3))
- ♻️ Rename test modules ([02ba2f4](https://github.com/8-bit-hunters/cargo-semantic-release/commit/02ba2f4ef574e3d4631d0f356e78ae34cf8f368c))
- ♻️ Make the SemanticVersionAction enum public ([4c0c377](https://github.com/8-bit-hunters/cargo-semantic-release/commit/4c0c377390f8386c8d70fab7f514613bbc0c717a))
- ♻️ Refactor finding the latest version tag with iterators ([ca37cdb](https://github.com/8-bit-hunters/cargo-semantic-release/commit/ca37cdb4e32c25690c09da5e87dd97030b440096))
- ♻️ Define commit fetcher extension for Repository ([02f8e8b](https://github.com/8-bit-hunters/cargo-semantic-release/commit/02f8e8bc917f7c41b406aa705378781d4099c8b9))
- ♻️ Define VersionTag extension for Repository ([74e4df0](https://github.com/8-bit-hunters/cargo-semantic-release/commit/74e4df081e923ac32452b951c8296aa64af5ba52))
- 🎨 Divide the library into modules ([440cd2d](https://github.com/8-bit-hunters/cargo-semantic-release/commit/440cd2d4030f43053f7127870cb7a830bf9bab0b))
- 🔧 Enable copyright display in the homepage footer ([03d71b2](https://github.com/8-bit-hunters/cargo-semantic-release/commit/03d71b228d20fd214fa71b57c974ae0b92b2e4e9))
- ♻️ Refactor test utils for less boilerplate code ([f39c4db](https://github.com/8-bit-hunters/cargo-semantic-release/commit/f39c4dbd8c6ba0b613fbb00a680371888781722d))
- ♻️ Refactor commit fetching ([2e5b2db](https://github.com/8-bit-hunters/cargo-semantic-release/commit/2e5b2dbfdab7d1c6ce436b70c3882b928368d4d1))
- 🧑‍💻 Add helper functions for testing tags ([f89b86e](https://github.com/8-bit-hunters/cargo-semantic-release/commit/f89b86e7bff9d19b65f8838d6c3360c465488e6e))
- 🔧 Add template chooser configuration ([6b5f98a](https://github.com/8-bit-hunters/cargo-semantic-release/commit/6b5f98ab539d995ba4f9ad86961ca54906e0203e))
- 🔧 Add issue template configuration ([dea70af](https://github.com/8-bit-hunters/cargo-semantic-release/commit/dea70af42f7210f1415a170074e8a754ce616e94))
- 🎨 Change structure of the code according to publicity ([216cc4d](https://github.com/8-bit-hunters/cargo-semantic-release/commit/216cc4d9d60c1655777454e97a31d288d8788a49))
- 👷 Change trigger method for Hugo site deploy ([0b8d36d](https://github.com/8-bit-hunters/cargo-semantic-release/commit/0b8d36da39916aec562e56800e3d3ddca54351b9))
- 🚨 Fix format issue ([0dab43c](https://github.com/8-bit-hunters/cargo-semantic-release/commit/0dab43cca40caf4a0a3e16a425344eaffbf8102f))
- 👷 Create workflow to do release with manual trigger ([dc46b71](https://github.com/8-bit-hunters/cargo-semantic-release/commit/dc46b7119a5efb556c935f81f66220f81e0173cf))

## Other

- 📝 Restructure `Development` pages on documentation site ([56dce03](https://github.com/8-bit-hunters/cargo-semantic-release/commit/56dce0398ccc717619511c4eabb4b27f40aa76e1))
- 💡 Add a comment on the clap parsers behaviour. ([8409907](https://github.com/8-bit-hunters/cargo-semantic-release/commit/84099078c9a7142d2ac454b4afb9442616def59a))
- 📝 Install the correct cargo-spellcheck crate. ([9929c05](https://github.com/8-bit-hunters/cargo-semantic-release/commit/9929c05e9924836657d565593c6d223483431aec))
- 📝 Add documentation for Display trait implementation ([7b0f9dc](https://github.com/8-bit-hunters/cargo-semantic-release/commit/7b0f9dc66d9fc22738650fdb2a3a717e25ba619b))
- 💡 Remove outdated TODO mark ([a01965f](https://github.com/8-bit-hunters/cargo-semantic-release/commit/a01965f2a97cac5dfe094be655ff78d6c909c4a1))
- 🔀 Merge pull request #14 from jw/gitmojis ([26e463f](https://github.com/8-bit-hunters/cargo-semantic-release/commit/26e463f7ec9c3eecb535f7a93da933fcc3fbb6b9))

# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### 0.1.3 (2025-02-24)


### Bug Fixes

* Fix clickhouse connection timeout ([a4d50a6](https://github.com/langdb/langdb-cloud/commit/a4d50a6a3a036822075b33d99d11e09c3f3e74ee))

### 0.1.2 (2025-02-21)


### Features

* Add api_invoke spans ([8398924](https://github.com/langdb/langdb-cloud/commit/83989242ebeb89626f95ba60e641cc48ddb81e1a))
* Add clickhouse dependency ([4e6ae44](https://github.com/langdb/langdb-cloud/commit/4e6ae44244d78baaaf4a1ca2db8d34e0d4aaf490))
* Add cost control and limit checker ([17eab2c](https://github.com/langdb/langdb-cloud/commit/17eab2cc5298f5421d2198bceb500bd5cf593010))
* Add database span writter ([94c048a](https://github.com/langdb/langdb-cloud/commit/94c048a3d6d30e44d69300b7cedb877a1a19e66a))
* Add extra to request ([a1ff5fb](https://github.com/langdb/langdb-cloud/commit/a1ff5fb71529350b5a1541f9d934a865f1373614))
* Add missing gemini parameters ([c22b37c](https://github.com/langdb/langdb-cloud/commit/c22b37cb4aef07ca82b9a4e95b8421270c022e49))
* Add model name and provider name to embeddings API ([e1d365f](https://github.com/langdb/langdb-cloud/commit/e1d365f31b58727c2c496ebdb41547d1bde27fa8))
* Add rate limiting ([459ba9d](https://github.com/langdb/langdb-cloud/commit/459ba9d4eb4ccaf8fbc2d4df696df85637320ea9))
* Add server crate with sample configuration ([a2e7c90](https://github.com/langdb/langdb-cloud/commit/a2e7c9025e9ca4116860916fbc183c97bccc89b4))
* Build for ubuntu and docker images ([#3](https://github.com/langdb/langdb-cloud/issues/3)) ([1e29aad](https://github.com/langdb/langdb-cloud/commit/1e29aad79853015760a7f2f06f7e9e993e60c8b2))
* display models ([8a1efdf](https://github.com/langdb/langdb-cloud/commit/8a1efdfc6e99a5728d5a962a7897f74a621c9d6d))
* Enable otel when clickhouse config provided ([5cdadb1](https://github.com/langdb/langdb-cloud/commit/5cdadb169502c1864f2a31588fc4ad4b1eb24e07))
* implement mcp support ([90220c2](https://github.com/langdb/langdb-cloud/commit/90220c289f5d37666002fd957d4cd0199013dac0))
* Implement tui ([#4](https://github.com/langdb/langdb-cloud/issues/4)) ([7589219](https://github.com/langdb/langdb-cloud/commit/758921962d9d2140b9814ad374f5e1e4ffc90d24))
* Improve UI ([#15](https://github.com/langdb/langdb-cloud/issues/15)) ([b83d183](https://github.com/langdb/langdb-cloud/commit/b83d18391dba63edbf2f14855f18b95513c15cb9))
* Integrate routed execution with fallbacks ([#20](https://github.com/langdb/langdb-cloud/issues/20)) ([3d75331](https://github.com/langdb/langdb-cloud/commit/3d75331cd49b4cb031371685539c3ff102f0d666))
* Print provider and model name in logs ([c8832e1](https://github.com/langdb/langdb-cloud/commit/c8832e1169c4c907ea19fe126ac8abdea8664f5e))
* Refactor targets usage for percentage router ([6d04e2d](https://github.com/langdb/langdb-cloud/commit/6d04e2d736ba8837e57de8b311c8eaf8baaf62b8))
* Support .env variables for config ([546d2a6](https://github.com/langdb/langdb-cloud/commit/546d2a66ab51263c857a7424570bddc8ad737271))
* Support langdb key ([#21](https://github.com/langdb/langdb-cloud/issues/21)) ([767e05e](https://github.com/langdb/langdb-cloud/commit/767e05e450b8d61bc345c0849feb20e6bf7dd07f))
* Support search in memory mcp tool ([#29](https://github.com/langdb/langdb-cloud/issues/29)) ([5d71a78](https://github.com/langdb/langdb-cloud/commit/5d71a783026ebad1eb3525b7ffd28be6ba8fb89f))
* Use in memory storage ([bf35718](https://github.com/langdb/langdb-cloud/commit/bf357181d34e02392444ddc465e880a720e9a4b8))
* Use time windows for metrics ([#28](https://github.com/langdb/langdb-cloud/issues/28)) ([c6ed8e4](https://github.com/langdb/langdb-cloud/commit/c6ed8e46dec5b25b88844e853960d39ab1034e1c))
* Use user in openai requests ([68415b0](https://github.com/langdb/langdb-cloud/commit/68415b015f4238ed942e4d5c293119c5fc6b995a))


### Bug Fixes

* Add router span ([1860f51](https://github.com/langdb/langdb-cloud/commit/1860f51b2874fa81e4117b35dc3e1f98f439413b))
* Create secure context for script router ([3cc7b8a](https://github.com/langdb/langdb-cloud/commit/3cc7b8affd6d9fe0190f4bab530eca5a33d15ca8))
* Fix connection to mcp servers ([e2208f8](https://github.com/langdb/langdb-cloud/commit/e2208f8d21eabe52e274e4b6777a6eee9cda0815))
* Fix gemini call when message is empty ([4a00a25](https://github.com/langdb/langdb-cloud/commit/4a00a258007ae175b33578df2e0b147c055c41e1))
* Fix langdb config load ([#26](https://github.com/langdb/langdb-cloud/issues/26)) ([8f02d58](https://github.com/langdb/langdb-cloud/commit/8f02d587a66ccf557290050a30ea2c16ed9d2745))
* Fix map tool names to labels in openai ([436d09e](https://github.com/langdb/langdb-cloud/commit/436d09e70b9ec907ee1c3a42a59b6f7e0561b9e4))
* Fix model name in models_call span ([62f5a38](https://github.com/langdb/langdb-cloud/commit/62f5a382228ee757b054f455ef75308cf5bf4b42))
* Fix provider name ([#18](https://github.com/langdb/langdb-cloud/issues/18)) ([7fdc24a](https://github.com/langdb/langdb-cloud/commit/7fdc24a883fa8462eed7d0512d76649f887c0b06))
* Fix provider name in tracing ([e779ec7](https://github.com/langdb/langdb-cloud/commit/e779ec76b49e9fae45ef14cf9a9826bb8e66a1ce))
* Fix response format usage ([#22](https://github.com/langdb/langdb-cloud/issues/22)) ([dbaf61d](https://github.com/langdb/langdb-cloud/commit/dbaf61d16d34a6a1747a982ad8d1ac7150963991))
* Fix routing direction for tps and requests metrics ([b96ee3e](https://github.com/langdb/langdb-cloud/commit/b96ee3ebf7cb03700442897371e1e12a001eeead))
* Fix serialization of user properties ([e8830c8](https://github.com/langdb/langdb-cloud/commit/e8830c82b405f73db6af3489a3238f84635a420f))
* Fix tags in tracing ([0b1ae3e](https://github.com/langdb/langdb-cloud/commit/0b1ae3ef2e473b52a132605313373dea6babddfd))
* Fix tonic shutdown on ctrl+c ([3c42dba](https://github.com/langdb/langdb-cloud/commit/3c42dba456ea566519cff5817d9d3bbf5ce40a7f))
* Fix tracing for openai and deepseek ([bbbae94](https://github.com/langdb/langdb-cloud/commit/bbbae94e7b2f0b89d12a6f00a07bf344857d044e))
* Improve error handling in loading config ([12a5cb2](https://github.com/langdb/langdb-cloud/commit/12a5cb26d94010f8c52f221dd9f5debea9c7f9bc))
* Return authorization error on invalid key ([000c376](https://github.com/langdb/langdb-cloud/commit/000c376db6c733fbc522050a2f3d9a9639b568d0))
* Return formated error on bedrock validation ([543585d](https://github.com/langdb/langdb-cloud/commit/543585d468514df3598a4def01ab985d6f802303))
* Store inference model name in model call span ([bbedf30](https://github.com/langdb/langdb-cloud/commit/bbedf300416edd5a7f39ade51065568b6e6716e9))

### 0.1.1 (2025-02-21)


### Features

* Add api_invoke spans ([8398924](https://github.com/langdb/ai-gateway/commit/83989242ebeb89626f95ba60e641cc48ddb81e1a))
* Add clickhouse dependency ([4e6ae44](https://github.com/langdb/ai-gateway/commit/4e6ae44244d78baaaf4a1ca2db8d34e0d4aaf490))
* Add cost control and limit checker ([17eab2c](https://github.com/langdb/ai-gateway/commit/17eab2cc5298f5421d2198bceb500bd5cf593010))
* Add database span writter ([94c048a](https://github.com/langdb/ai-gateway/commit/94c048a3d6d30e44d69300b7cedb877a1a19e66a))
* Add extra to request ([a1ff5fb](https://github.com/langdb/ai-gateway/commit/a1ff5fb71529350b5a1541f9d934a865f1373614))
* Add missing gemini parameters ([c22b37c](https://github.com/langdb/ai-gateway/commit/c22b37cb4aef07ca82b9a4e95b8421270c022e49))
* Add model name and provider name to embeddings API ([e1d365f](https://github.com/langdb/ai-gateway/commit/e1d365f31b58727c2c496ebdb41547d1bde27fa8))
* Add rate limiting ([459ba9d](https://github.com/langdb/ai-gateway/commit/459ba9d4eb4ccaf8fbc2d4df696df85637320ea9))
* Add server crate with sample configuration ([a2e7c90](https://github.com/langdb/ai-gateway/commit/a2e7c9025e9ca4116860916fbc183c97bccc89b4))
* Build for ubuntu and docker images ([#3](https://github.com/langdb/ai-gateway/issues/3)) ([1e29aad](https://github.com/langdb/ai-gateway/commit/1e29aad79853015760a7f2f06f7e9e993e60c8b2))
* display models ([8a1efdf](https://github.com/langdb/ai-gateway/commit/8a1efdfc6e99a5728d5a962a7897f74a621c9d6d))
* Enable otel when clickhouse config provided ([5cdadb1](https://github.com/langdb/ai-gateway/commit/5cdadb169502c1864f2a31588fc4ad4b1eb24e07))
* implement mcp support ([90220c2](https://github.com/langdb/ai-gateway/commit/90220c289f5d37666002fd957d4cd0199013dac0))
* Implement tui ([#4](https://github.com/langdb/ai-gateway/issues/4)) ([7589219](https://github.com/langdb/ai-gateway/commit/758921962d9d2140b9814ad374f5e1e4ffc90d24))
* Improve UI ([#15](https://github.com/langdb/ai-gateway/issues/15)) ([b83d183](https://github.com/langdb/ai-gateway/commit/b83d18391dba63edbf2f14855f18b95513c15cb9))
* Integrate routed execution with fallbacks ([#20](https://github.com/langdb/ai-gateway/issues/20)) ([3d75331](https://github.com/langdb/ai-gateway/commit/3d75331cd49b4cb031371685539c3ff102f0d666))
* Print provider and model name in logs ([c8832e1](https://github.com/langdb/ai-gateway/commit/c8832e1169c4c907ea19fe126ac8abdea8664f5e))
* Refactor targets usage for percentage router ([6d04e2d](https://github.com/langdb/ai-gateway/commit/6d04e2d736ba8837e57de8b311c8eaf8baaf62b8))
* Support .env variables for config ([546d2a6](https://github.com/langdb/ai-gateway/commit/546d2a66ab51263c857a7424570bddc8ad737271))
* Support langdb key ([#21](https://github.com/langdb/ai-gateway/issues/21)) ([767e05e](https://github.com/langdb/ai-gateway/commit/767e05e450b8d61bc345c0849feb20e6bf7dd07f))
* Support search in memory mcp tool ([#29](https://github.com/langdb/ai-gateway/issues/29)) ([5d71a78](https://github.com/langdb/ai-gateway/commit/5d71a783026ebad1eb3525b7ffd28be6ba8fb89f))
* Use in memory storage ([bf35718](https://github.com/langdb/ai-gateway/commit/bf357181d34e02392444ddc465e880a720e9a4b8))
* Use time windows for metrics ([#28](https://github.com/langdb/ai-gateway/issues/28)) ([c6ed8e4](https://github.com/langdb/ai-gateway/commit/c6ed8e46dec5b25b88844e853960d39ab1034e1c))
* Use user in openai requests ([68415b0](https://github.com/langdb/ai-gateway/commit/68415b015f4238ed942e4d5c293119c5fc6b995a))


### Bug Fixes

* Add router span ([1860f51](https://github.com/langdb/ai-gateway/commit/1860f51b2874fa81e4117b35dc3e1f98f439413b))
* Create secure context for script router ([3cc7b8a](https://github.com/langdb/ai-gateway/commit/3cc7b8affd6d9fe0190f4bab530eca5a33d15ca8))
* Fix connection to mcp servers ([e2208f8](https://github.com/langdb/ai-gateway/commit/e2208f8d21eabe52e274e4b6777a6eee9cda0815))
* Fix langdb config load ([#26](https://github.com/langdb/ai-gateway/issues/26)) ([8f02d58](https://github.com/langdb/ai-gateway/commit/8f02d587a66ccf557290050a30ea2c16ed9d2745))
* Fix map tool names to labels in openai ([436d09e](https://github.com/langdb/ai-gateway/commit/436d09e70b9ec907ee1c3a42a59b6f7e0561b9e4))
* Fix model name in models_call span ([62f5a38](https://github.com/langdb/ai-gateway/commit/62f5a382228ee757b054f455ef75308cf5bf4b42))
* Fix provider name ([#18](https://github.com/langdb/ai-gateway/issues/18)) ([7fdc24a](https://github.com/langdb/ai-gateway/commit/7fdc24a883fa8462eed7d0512d76649f887c0b06))
* Fix provider name in tracing ([e779ec7](https://github.com/langdb/ai-gateway/commit/e779ec76b49e9fae45ef14cf9a9826bb8e66a1ce))
* Fix response format usage ([#22](https://github.com/langdb/ai-gateway/issues/22)) ([dbaf61d](https://github.com/langdb/ai-gateway/commit/dbaf61d16d34a6a1747a982ad8d1ac7150963991))
* Fix routing direction for tps and requests metrics ([b96ee3e](https://github.com/langdb/ai-gateway/commit/b96ee3ebf7cb03700442897371e1e12a001eeead))
* Fix serialization of user properties ([e8830c8](https://github.com/langdb/ai-gateway/commit/e8830c82b405f73db6af3489a3238f84635a420f))
* Fix tags in tracing ([0b1ae3e](https://github.com/langdb/ai-gateway/commit/0b1ae3ef2e473b52a132605313373dea6babddfd))
* Fix tonic shutdown on ctrl+c ([3c42dba](https://github.com/langdb/ai-gateway/commit/3c42dba456ea566519cff5817d9d3bbf5ce40a7f))
* Fix tracing for openai and deepseek ([bbbae94](https://github.com/langdb/ai-gateway/commit/bbbae94e7b2f0b89d12a6f00a07bf344857d044e))
* Improve error handling in loading config ([12a5cb2](https://github.com/langdb/ai-gateway/commit/12a5cb26d94010f8c52f221dd9f5debea9c7f9bc))
* Return authorization error on invalid key ([000c376](https://github.com/langdb/ai-gateway/commit/000c376db6c733fbc522050a2f3d9a9639b568d0))
* Store inference model name in model call span ([bbedf30](https://github.com/langdb/ai-gateway/commit/bbedf300416edd5a7f39ade51065568b6e6716e9))

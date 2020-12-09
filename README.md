# custom_hint_section

Adds a custom `branchHints` section to a wasm module, adding to all `if` and `br_if` instructions a hint that the condition is likely false.

For the section to be useful, you need a patched version of V8: https://github.com/yuri91/v8/tree/custom_section

See https://github.com/WebAssembly/branch-hinting/issues/1 for more infos.

## Usage

```
npm install
node main.js test.wasm test_hinted.wasm
```

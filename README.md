# custom_hint_section

Adds a custom `branchHints` section to a wasm module, setting all `if`s to likely false and all `br_if`s to likely true.

For the section to be useful, you need a patched version of V8: https://github.com/yuri91/v8/tree/custom_section

See https://github.com/WebAssembly/branch-hinting/issues/1 for more infos.

## Usage

```
npm install
node main.js test.wasm test_hinted.wasm
```

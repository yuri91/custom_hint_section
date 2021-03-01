# custom_hint_section

Adds a custom `branchHints` section to a wasm module.

For the section to be useful, you need a patched version of V8: https://github.com/yuri91/v8/tree/custom_section

See https://github.com/WebAssembly/branch-hinting/ for more infos on the branch hinting proposal

## Usage

```
cargo run test.wasm test_hint.wasm hints.txt
```

This command adds a branchHints section to `test.wasm` and outputs the resulting module
in `test_hint.wasm`.

The section is compiled based on the contents of `hints.txt`, which has this format:

```
<f_index>
	<branch_offset> <hint>
	<branch_offset> <hint>
	...
<f_index>
	<branch_offset> <hint>
	<branch_offset> <hint>
	...
```

Where `f_index` is the index of the function for which the following hints apply.
Hints for a function are indented with a single tab, one per line.

`branch_offset` is the byte offset of the branch instruction from the first instruction
of the function.

`hint` is `0` if the branch is unlikely or `1` if it is likely.

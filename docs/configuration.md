# Configuration

Use `.alofmt.toml` to define formatting policy for a project. alofmt applies
settings in this order:

1. Start with the project-neutral defaults.
2. Load the nearest `.alofmt.toml`, or the file passed to `--config`.
3. Apply CLI style options.

Pass `--no-config` to use the defaults. Unknown keys and invalid values are
errors.

## Options

Every field is optional.

| Option | Default | Purpose |
| --- | --- | --- |
| `line_width` | `80` | Maximum width used to choose line breaks. |
| `indent_width` | `2` | Spaces emitted for one indentation level. |
| `fit_indent_width` | `2` | Indentation width used while measuring whether a group fits. |
| `quote_style` | `"preserve"` | Plain string and symbol delimiters: `single`, `double`, or `preserve`. |
| `trailing_commas` | `false` | Add trailing commas to broken collections and argument lists. |
| `prefer_percent_arrays` | `false` | Convert eligible arrays to `%w` and `%i` literals. |
| `normalize_number_separators` | `false` | Add thousands separators to eligible decimal integers. |
| `explicit_standard_error` | `false` | Spell an omitted rescue class as `StandardError`. |
| `ignore_directives` | `["alofmt-ignore"]` | Comment bodies that copy the following node verbatim. |
| `chain_break_threshold` | `3` | Number of chained calls that makes a chain break. |
| `compact_chain_break_threshold` | `3` | Chain threshold inside configured compact-chain blocks. |
| `compact_chain_blocks` | `[]` | Block-call names that use the compact threshold. |
| `unaligned_command_calls` | `[]` | Command-call names whose broken arguments do not align under the first argument. |
| `max_command_alignment` | `40` | Longest command prefix eligible for continuation alignment; `0` disables it. |

Quote conversion occurs only when changing delimiters preserves the string's
meaning.

Most projects should keep `indent_width` and `fit_indent_width` equal. Separate
values support compatibility with layouts that measure and emit indentation
differently.

## Policy hooks

`ignore_directives` matches the body of a comment. With the default setting,
this copies the next node without formatting it:

```ruby
# alofmt-ignore
call(   left, right   )
```

`compact_chain_blocks` applies a separate chain threshold inside selected
block calls. `unaligned_command_calls` selects command calls whose continued
arguments start at the call's indentation.

The corresponding CLI flags are repeatable:

```sh
alofmt --check \
  --compact-chain-block sig \
  --unaligned-command-call to \
  .
```

Providing one of these flags replaces the configured list. Use
`--no-ignore-directives`, `--no-compact-chain-blocks`, or
`--no-unaligned-command-calls` to clear a list.

Run `alofmt --help` for all scalar CLI overrides.

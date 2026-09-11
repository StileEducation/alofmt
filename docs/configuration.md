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
| `block_delimiters` | `"line_count_based"` | Block delimiters where either parses the same: `line_count_based` uses `{ }` when the block fits and `do`/`end` when it breaks, `always_braces` and `always_do_end` fix one, `preserve` keeps the source's. |
| `percent_arrays` | `"preserve"` | Arrays of single-word strings or symbols: `prefer` collapses bracketed ones to `%w` and `%i`, `avoid` rewrites `%w` and `%i` literals as bracketed arrays, `preserve` keeps every array as written. |
| `normalize_number_separators` | `false` | Add thousands separators to eligible decimal integers. |
| `explicit_standard_error` | `false` | Spell an omitted rescue class as `StandardError`. |
| `ignore_directives` | `["alofmt-ignore"]` | Comment bodies that copy the following node verbatim. |
| `chain_break_threshold` | `3` | Number of chained calls that makes a chain break. |
| `compact_chain_break_threshold` | `3` | Chain threshold inside configured compact-chain blocks. |
| `compact_chain_blocks` | `[]` | Block-call names that use the compact threshold. |
| `unaligned_command_calls` | `[]` | Command-call names whose broken arguments do not align under the first argument. |
| `max_command_alignment` | `40` | Longest command prefix eligible for continuation alignment; `0` disables it. |
| `delimited_argument_alignment` | `"aligned"` | Where a command call's sole bracketed argument sits when the call breaks: `aligned` under the argument's own column, or `consistent` one level in from the start of the line with the closing bracket back at the line's indent. |
| `multiline_assignment_layout` | `"new_line"` | Where the value of an assignment that cannot fit goes, named after RuboCop's styles: `new_line` moves it down whole, `same_line` keeps a breakable value's head beside the operator. |
| `method_call_with_args_parentheses` | `"preserve"` | Parentheses around arguments: `require_parentheses` on every call, `super` and `yield` with arguments; `omit_parentheses` drops them where the call parses the same; `preserve` as written. |
| `method_call_without_args_parentheses` | `"preserve"` | Empty parentheses on a call with no arguments: `require_parentheses` for `()` on member calls with no block; `omit_parentheses` drops `()` where the bare name reads as the same call; `preserve` as written. |
| `redundant_self` | `"preserve"` | A `self.` receiver: `require_self` adds it to member calls; `omit_self` drops it where the bare name reads as the same call; `preserve` as written. |
| `allowed_methods` | `[]` | Method names exempt from the three call policies. `super` and `yield` are accepted. |
| `member_macros` | `["attr", "attr_reader", "attr_accessor", "define_method"]` | Calls whose positional symbol and string arguments name members, alongside `def` and `alias`. |

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

## Call policies

`method_call_with_args_parentheses`, `method_call_without_args_parentheses`
and `redundant_self` are named after the corresponding RuboCop cops, and the
values are the RuboCop style names. `require_parentheses` on calls without
arguments and `require_self` have no RuboCop equivalent. `require_self`
requires Ruby 2.7 or later, where a private method can be called with a
literal `self` receiver. Added parentheses or `self.` never change which
method is called, so a misclassified call is a style difference, never a
behaviour difference.

A **member call** is a call with no receiver, or `self` as the receiver,
to a method defined in the lexically enclosing class, module or top-level
body at the same `self` level as the call: by a `def`, an `alias`, or a
positional symbol or string argument of a `member_macros` call. Calls inside
`def self.x` and `class << self` bodies are matched against singleton
definitions; calls inside instance method bodies against instance
definitions. Definitions in superclasses, included modules and other
reopenings of the class are not visible, so calls to them stay as written.
For `T::Struct`, add `const` and `prop` to `member_macros`.

A **macro** is a receiverless call that is a statement of a class, module,
singleton-class or top-level body, or of a block attached to a macro:
`attr_reader :x`, `include Foo`, `private def ...`, `describe ... do` and the
statements inside it. `allowed_methods` are exempt from every call policy;
macros are exempt from the `require` policies.

The `omit` values remove syntax only where the bare form parses the same,
so each keeps the source's spelling in a fixed set of cases:

- `redundant_self = "omit_self"` keeps `self.` on setters and operators
  (`self.x = 1`, `self[0]`, `self + 1`), keyword-named methods (`self.class`,
  `self.then`), constant-like names (`self.Foo`), the implicit block
  parameter `it`, and any name that is also a local variable in scope.
- `method_call_without_args_parentheses = "omit_parentheses"` drops `()`
  from any call with no arguments, keeping it for constant-like names
  (`Integer()`), `.()`, and a name that is also a local variable in scope.
  `super()` is never touched, since it differs from bare `super`.
- `method_call_with_args_parentheses = "omit_parentheses"` drops the
  parentheses of a call, `super` or `yield` in **tail position** only: a
  statement; the value of an assignment, or the sole value of `return`,
  `break` or `next`, that is itself a statement; or the expression of a
  rescue modifier. A call that is a receiver, an operand, an argument, an
  element, a ternary branch, a predicate, inside string interpolation,
  inside parentheses or the value of an endless `def` keeps them. So does a
  call with a block, a call with a block anywhere in its arguments, a call
  with a comment inside the parentheses, a call whose first argument starts
  with `-`, `+`, `*`, `&`, `[`, `(`, `/`, `%`,
  `?`, `::` or `{`, a call whose arguments include a keyword expression, an
  assignment, `and`, `or`, `not`, `...` or a value-omitted label (`b:`), and
  a call on a name that is also a local variable in scope. Dropped
  parentheses return when the call has to break across lines, as with
  `yield a, b`: `foo a, b` when it fits, `foo(\n  a,\n  b\n)` when not. A
  call written without parentheses keeps the command layout either way.

```toml
method_call_with_args_parentheses = "require_parentheses"
method_call_without_args_parentheses = "require_parentheses"
redundant_self = "require_self"
allowed_methods = ["raise", "to", "not_to"]
member_macros = ["attr", "attr_reader", "attr_accessor", "define_method", "const", "prop"]
```

With that configuration:

```ruby
class Job
  attr_reader :queue

  def run
    queue.push build          # self.queue().push(self.build())
    puts status               # puts(status)
    raise ArgumentError, 'x'  # unchanged: raise is allowed
    helper 1 do |x|           # self.helper(1) { |x| x }
      x
    end
  end

  def build
  end

  def helper(x)
  end
end
```

A command call whose only argument is a conditional is printed as
`foo(a ? b : c)`. Once a command call has parentheses it is an ordinary
chain member, and any block on it follows `block_delimiters` like the block
of any parenthesised call. Under `require_self` or `omit_self`, a literal
`self` receiver is never a chain link: `self.a.b.c` counts two chained
calls, the same as `a.b.c`, whichever spelling the source has.

The CLI flags are `--method-call-with-args-parentheses`,
`--method-call-without-args-parentheses` and `--redundant-self`, plus the
repeatable `--allowed-method` and `--member-macro` with
`--no-allowed-methods` and `--no-member-macros` to clear each list.

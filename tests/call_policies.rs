//! The parentheses and `self.` policies, each shown rewriting source it
//! applies to and leaving alone the source it must not touch.

use alofmt::FormatOptions;

fn format(profile: &str, source: &str) -> String {
    let options = FormatOptions::from_toml(profile).expect("valid profile");
    alofmt::format_with_options(source.as_bytes(), &options).expect("formats")
}

const WITH_ARGS: &str = r#"
    method_call_with_args_parentheses = "require_parentheses"
    allowed_methods = ["raise", "to"]
"#;

const WITHOUT_ARGS: &str = r#"
    method_call_without_args_parentheses = "require_parentheses"
    member_macros = ["attr_reader", "const"]
"#;

const REQUIRE_SELF: &str = r#"
    redundant_self = "require_self"
"#;

#[test]
fn require_parentheses_wraps_every_argument_list() {
    let source = "\
def run
  foo a, b
  x.foo a
  foo &block
  foo a ? b : c
  foo a do |x|
    x
  end
  super a
  yield a
  x.y = 1
  a + b
  foo bar baz
end
";
    let expected = "\
def run
  foo(a, b)
  x.foo(a)
  foo(&block)
  foo(a ? b : c)
  foo(a) { |x| x }
  super(a)
  yield(a)
  x.y = 1
  a + b
  foo(bar(baz))
end
";
    assert_eq!(format(WITH_ARGS, source), expected);
}

#[test]
fn require_parentheses_leaves_macros_and_allowed_methods() {
    let source = "\
require 'json'
puts 'top level'

class Job
  include Comparable
  attr_reader :name
  private def secret
    raise ArgumentError, 'm'
    puts 'body'
  end
end

describe Job do
  it 'runs' do
    expect(job).to eq 1
    make_thing 'x'
  end
end
";
    let expected = "\
require 'json'
puts 'top level'

class Job
  include Comparable
  attr_reader :name
  private def secret
    raise ArgumentError, 'm'
    puts('body')
  end
end

describe Job do
  it 'runs' do
    expect(job).to eq(1)
    make_thing 'x'
  end
end
";
    assert_eq!(format(WITH_ARGS, source), expected);
}

#[test]
fn require_empty_parentheses_marks_member_calls_only() {
    let source = "\
class Job
  attr_reader :queue
  const :status, String
  alias title name

  def run
    queue
    status
    title
    name
    self.name
    build
    build { 1 }
    format
    verbose?
    build.upcase
    name = 1
    name
  end

  def build
  end

  def name
  end

  def self.count
    total
    build
  end

  def self.total
  end

  class << self
    def registry
      total
    end
  end
end

module Helpers
  def helper
    other
  end

  def other
  end
end
";
    let expected = "\
class Job
  attr_reader :queue
  const :status, String
  alias title name

  def run
    queue()
    status()
    title()
    name()
    self.name()
    build()
    build { 1 }
    format
    verbose?
    build().upcase
    name = 1
    name
  end

  def build
  end

  def name
  end

  def self.count
    total()
    build
  end

  def self.total
  end

  class << self
    def registry
      total()
    end
  end
end

module Helpers
  def helper
    other()
  end

  def other
  end
end
";
    assert_eq!(format(WITHOUT_ARGS, source), expected);
}

#[test]
fn require_self_prefixes_member_calls_outside_macro_position() {
    let source = "\
class Job
  attr_reader :name
  helper name

  def run
    name
    puts name
    build 1
    build(1).upcase
    self.name
    name = 1
    name
    other
  end

  def build(x)
  end

  def self.count
    name
  end

  class Inner
    def go
      name
    end
  end
end
";
    let expected = "\
class Job
  attr_reader :name
  helper name

  def run
    self.name
    puts self.name
    self.build 1
    self.build(1).upcase
    self.name
    name = 1
    name
    other
  end

  def build(x)
  end

  def self.count
    name
  end

  class Inner
    def go
      name
    end
  end
end
";
    assert_eq!(format(REQUIRE_SELF, source), expected);
}

#[test]
fn policies_compose_into_self_dot_call_with_empty_parentheses() {
    let profile = r#"
        method_call_with_args_parentheses = "require_parentheses"
        method_call_without_args_parentheses = "require_parentheses"
        redundant_self = "require_self"
    "#;
    let source = "\
class Foo
  def method_a
    method_b
    method_c 1
    puts \"hi\"
  end

  def method_b
    puts \"hi\"
  end

  def method_c(x)
  end
end
";
    let expected = "\
class Foo
  def method_a
    self.method_b()
    self.method_c(1)
    puts(\"hi\")
  end

  def method_b
    puts(\"hi\")
  end

  def method_c(x)
  end
end
";
    assert_eq!(format(profile, source), expected);
}

#[test]
fn preserve_leaves_member_calls_untouched() {
    let source = "\
class Foo
  def method_a
    method_b
    method_c 1
  end

  def method_b
  end

  def method_c(x)
  end
end
";
    assert_eq!(format("", source), source);
}

#[test]
fn require_self_does_not_lengthen_chains() {
    // `self.` is added on the first pass and read back on the second; the
    // chain must count the same both times or the second pass breaks it.
    let profile = r#"
        redundant_self = "require_self"
        method_call_without_args_parentheses = "require_parentheses"
        chain_break_threshold = 3
    "#;
    let source = "\
class Report
  def print
    summaries.sort_by { |_, summary| -summary[:total_time] }.first(count).each { |x| x }
  end

  def summaries
  end
end
";
    let expected = "\
class Report
  def print
    self.summaries()
      .sort_by { |_, summary| -summary[:total_time] }
      .first(count)
      .each { |x| x }
  end

  def summaries
  end
end
";
    let once = format(profile, source);
    assert_eq!(once, expected);
    assert_eq!(format(profile, &once), expected);
}

const OMIT_SELF: &str = r#"
    redundant_self = "omit_self"
"#;

const OMIT_ALL: &str = r#"
    method_call_with_args_parentheses = "omit_parentheses"
    method_call_without_args_parentheses = "omit_parentheses"
    redundant_self = "omit_self"
"#;

#[test]
fn omit_self_keeps_it_where_the_bare_name_reads_differently() {
    let source = "\
class Thing
  def go(count)
    self.items
    self.build(1)
    self.each { |x| x }
    self.class
    self.then { |x| x }
    self.Integer
    self.count
    self.flag = true
    self.flag += 1
    self[0]
    self + 1
    self&.build
    [1].each { self.it }
    self.puts 'x'
  end
end
";
    let expected = "\
class Thing
  def go(count)
    items
    build(1)
    each { |x| x }
    self.class
    self.then { |x| x }
    self.Integer
    self.count
    self.flag = true
    self.flag += 1
    self[0]
    self + 1
    self&.build
    [1].each { self.it }
    puts 'x'
  end
end
";
    assert_eq!(format(OMIT_SELF, source), expected);
}

#[test]
fn omit_self_does_not_lengthen_chains() {
    let profile = r#"
        redundant_self = "omit_self"
        chain_break_threshold = 3
    "#;
    let source = "\
class Report
  def print
    self.summaries.sort_by { |_, summary| -summary[:total_time] }.first(count).each { |x| x }
  end
end
";
    let expected = "\
class Report
  def print
    summaries
      .sort_by { |_, summary| -summary[:total_time] }
      .first(count)
      .each { |x| x }
  end
end
";
    let once = format(profile, source);
    assert_eq!(once, expected);
    assert_eq!(format(profile, &once), expected);
}

#[test]
fn omit_empty_parentheses_keeps_constants_locals_and_super() {
    let profile = r#"
        method_call_without_args_parentheses = "omit_parentheses"
    "#;
    let source = "\
def go(count)
  items()
  x.items()
  items().upcase
  items() { 1 }
  Integer()
  x.()
  count()
  [1].each { it() }
  super()
  yield()
  count = 1
end
";
    let expected = "\
def go(count)
  items
  x.items
  items.upcase
  items { 1 }
  Integer()
  x.()
  count()
  [1].each { it() }
  super()
  yield()
  count = 1
end
";
    assert_eq!(format(profile, source), expected);
}

#[test]
fn omit_parentheses_drops_them_in_tail_position_only() {
    let profile = r#"
        method_call_with_args_parentheses = "omit_parentheses"
    "#;
    let source = "\
def go(count)
  build(1)
  x.build(1)
  x = build(1)
  @y ||= build(1)
  a, b = build(1)
  return build(1) if x
  return build(1), 2
  build(1) rescue nil
  build(1).upcase
  build(1) + 1
  !build(1)
  [build(1)]
  p(build(1))
  x ? build(1) : 2
  \"#{build(1)}\"
  (build(1))
  if build(1)
    x
  end
  case status = build(1)
  when 1 then x
  end
  count(1)
  super(1)
  yield(1)
  x = yield(1)
  [yield(1)]
end

def endless = build(1)
";
    let expected = "\
def go(count)
  build 1
  x.build 1
  x = build 1
  @y ||= build 1
  a, b = build 1
  return build 1 if x
  return build(1), 2
  begin
    build 1
  rescue
    nil
  end
  build(1).upcase
  build(1) + 1
  !build(1)
  [build(1)]
  p build(1)
  x ? build(1) : 2
  \"#{build(1)}\"
  (build(1))
  x if build(1)
  case status = build(1)
  when 1
    x
  end
  count(1)
  super 1
  yield 1
  x = yield 1
  [yield(1)]
end

def endless = build(1)
";
    assert_eq!(format(profile, source), expected);
}

#[test]
fn omit_parentheses_keeps_them_for_ambiguous_arguments() {
    let profile = r#"
        method_call_with_args_parentheses = "omit_parentheses"
    "#;
    let source = "\
def go
  build(-1)
  build(*items)
  build(&blk)
  build(1, &blk)
  build([1])
  build((1))
  build(/re/)
  build(%w[a])
  build(::A)
  build({ a: 1 })
  build(a: 1)
  build(1) { |x| x }
  build(x = 1)
  build(a && b)
  build(not a)
  build(a ? b : c)
  build(principal, capabilities:)
  build(
    1, # comment
    2
  )
end
";
    let expected = "\
def go
  build(-1)
  build(*items)
  build(&blk)
  build 1, &blk
  build([1])
  build((1))
  build(/re/)
  build(%w[a])
  build(::A)
  build({ a: 1 })
  build a: 1
  build(1) { |x| x }
  build(x = 1)
  build(a && b)
  build(not a)
  build(a ? b : c)
  build(principal, capabilities:)
  build(
    1, # comment
    2
  )
end
";
    assert_eq!(format(profile, source), expected);
}

#[test]
fn omitted_parentheses_return_when_the_call_breaks() {
    // The broken form keeps the parentheses, as `yield` does. A block among
    // the arguments keeps them in every form: flat, a `do` block would bind
    // to the outer call, and a brace block may become a `do` block.
    let profile = r#"
        method_call_with_args_parentheses = "omit_parentheses"
    "#;
    let source = "\
def go
  build(a_very_long_first_argument_name, a_very_long_second_argument_name, third_one)
  build(checked_out: Array.new(count) do
    x
  end, deleted: [])
  build(items.map { |x| x })
end
";
    let expected = "\
def go
  build(
    a_very_long_first_argument_name,
    a_very_long_second_argument_name,
    third_one
  )
  build(checked_out: Array.new(count) { x }, deleted: [])
  build(items.map { |x| x })
end
";
    let once = format(profile, source);
    assert_eq!(once, expected);
    assert_eq!(format(profile, &once), expected);
}

#[test]
fn omit_policies_compose_and_stay_fixed() {
    let source = "\
class Foo
  def method_a
    self.method_b()
    self.method_c(1)
    puts(\"hi\")
  end

  def method_b
    puts(\"hi\")
  end

  def method_c(x)
  end
end
";
    let expected = "\
class Foo
  def method_a
    method_b
    method_c 1
    puts \"hi\"
  end

  def method_b
    puts \"hi\"
  end

  def method_c(x)
  end
end
";
    let once = format(OMIT_ALL, source);
    assert_eq!(once, expected);
    assert_eq!(format(OMIT_ALL, &once), expected);
}

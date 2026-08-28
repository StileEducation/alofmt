def foo = bar
def foo(a) = bar
def self.foo = 1
def foo.bar = 1
def ==(other) = 1
def foo(a) = puts a
def foo(a) = puts a, b
def foo(a, b) = puts(a)
def foo = bar { x }
def foo = bar { x }
def foo = 1 # trailing
def foo(a) = 1 # trailing
def foo = 1 + 2
def foooooooooooooooooooooooooooooooooooooooooooo(aaaaaaaaaaaaaaaaaaaaaaa) =
    bbbbbbbbbbbbbbbbbbbbbbb
def foooooooooooooooooooooooooooooooooooooooooooo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
) = bbbbbbbbbbbbbbbbbbbbbbb(1, 2)
def foooooooooooooooooooooooooooooooooooo =
    bar(aaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbb)
def foooooooooooooooooooooooooooooooooooo =
    bar(aaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbb)
def foo =
    bar(
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
    )
def foo =
    [
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
    ]
def foo =
    foo
        .bar
        .baz
        .aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        .bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
        .cccccccccccccccccccc
def to_s = "#{a}-#{b}"

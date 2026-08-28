def foo(a)
end
def foo(a, b)
end
def foo(a = 1)
end
def foo(a, b = 1, c = {}, d = [])
end
def foo(*rest)
end
def foo(*)
end
def foo(a, *)
end
def foo(a, *b, c)
end
def foo(**kw)
end
def foo(**)
end
def foo(a, **)
end
def foo(**nil)
end
def foo(a:)
end
def foo(a: 1, b: nil, c: 'x')
end
def foo(&blk)
end
def foo(&)
end
def foo(a, &)
end
def foo(...)
end
def foo(a, ...)
end
def foo(*, **, &)
end
def foo(a, b = 1, *c, d, e:, f: 2, **g, &h)
end
def foo(a, b = 1, *c, d:, e: 2, **f, &g)
end
def foo(a, (b, c), d)
end
def foo((b, *c), (d, (e, f)))
end
def foo(a, (b, c, *), d)
end
def foo(a, (*, b), d)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    ddddd
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    dddd
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    *ddddd
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    &ddddd
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    **ddddd
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    ddddd:
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    ddddd: 1
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    dddddddd = 1
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    ...
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccccccc,
    **nil
)
end
def self.foo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
)
end
def foo(
    a = bar(
        aaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccccccccccccccc,
    ),
    b
)
end
def foo(
    a: bar(
        aaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccccc,
        ddddddd,
        eeeeeeeeeeeeeeeeeeeeee,
    ),
    b:
)
end
def foo(
    a: [
        aaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccccc,
        ddddddd,
        eeeeeeeeeeeeeeeeeeeeee,
    ],
    b:
)
end
def foo(
    a: { a: 1 },
    b: {
        aaaaaaaaaaaaaaaaaaaaaaa: 1,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbb: 2,
        cccccccccccccccccccccccc: 3,
    }
)
end
def foo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    (bbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccccccccccccc)
)
end
def foo(
    (
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccccccccccccc
    ),
    d
)
end
def foo(
    a, # comment a
    b
) # comment b
end
def foo(
    a, # comment a
    b
)
end
def foo(
    a,
    # own line
    b
)
end
def foo(a) # trailing
end
def foo(a, b)
    x
end

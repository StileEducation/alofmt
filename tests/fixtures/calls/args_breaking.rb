foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    e: 1,
    f: 2,
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    { e: 1, f: 2 },
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bar(
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeee,
    ),
)
foo(
    bar(
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeee,
        ffffffffffff,
    ),
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    [
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeeeee,
    ],
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    &blk
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    *dddddddddddddddd,
    **e,
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
) { x }
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
) do
    x
    y
end
foo(
    aaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbb + cccccccccccccccccccccc +
        dddddddddddddddd + eeeeeee,
)
foo(
    1,
    2,
    aaaaaaaaaaaaaaaaaaaa
        .bbbbbbbbbbbbbbbbbbbbbbbb
        .cccccccccccccccccccccc
        .dddddddddddddddd
        .eeeeeeee,
)
foo(
    bar(1),
    baz(2),
    aaaaaaaaaaaaaaaaaaaa
        .bbbbbbbbbbbbbbbbbbbbbbbb
        .cccccccccccccccccccccc
        .ddddddddd(1),
    2,
)
foo(
    [
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeee,
    ],
)
foo(
    {
        aaaaaaaaaaaaaaaaaaaa: 1,
        bbbbbbbbbbbbbbbbbbbbbbbb: 2,
        cccccccccccccccccccccc: 3,
        dddddddddd: 4,
    },
)
foo(
    aaaaaaaaaaaaaaaaaaaa: 1,
    bbbbbbbbbbbbbbbbbbbbbbbb: 2,
    cccccccccccccccccccccc: 3,
    ddddddddddddd: 4,
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    e: {
        f: 1,
    },
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    e: [1, 2],
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    e: {
        f: {
            g: 1,
        },
    },
)
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    **{ f: 1 },
)
foo(
    aaaaaaaaaaaaaaaaaaaa:
        bbbbbbbbbbbbbbbbbbbbbbbbccccccccccccccccccccccddddddddddddddddeeeeeeeeeeeefffffffffff,
)
foo(
    aaaaaaaaaaaaaaaaaaaa:
        foo(
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd,
            eeeeeeeeeeee,
        ),
)
foo(
    bar do
        x
        y
    end,
)
foo(
    1,
    bar do
        x
        y
    end,
)
foo(
    bar do
        x
        y
    end,
    1,
)
foo(*args)
foo(**opts)
foo(&blk)
foo(1, **opts, &blk)
foo(*a, b)
foo(1, { a: 1 })
foo({ a: 1 }, 2)
foo(
    aaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    ccccccccccc,
).bar(2)
foo(
    aaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    ccccccccccc,
).bar(2).baz
foo.bar(
    aaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    ccccccccccc,
).baz(2)

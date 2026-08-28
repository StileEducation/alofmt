def m
    super
    super()
    super(1, 2)
    super 1
    super a: 1
    super(&blk)
    super { x }
    super do
        x
        y
    end
    super(1) do x end
    super(1) do end
    super 1 do x end
    super aaaaaaaaaaaaaaaaaaaa,
                bbbbbbbbbbbbbbbbbbbbbbbb,
                cccccccccccccccccccccc,
                dddddddddddddddd,
                eeeeeeeeeee
    super(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeeee,
    )
    super(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddd,
        &blk
    )
    super(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddd,
    ) do x end
    super foo(
                    aaaaaaaaaaaaaaaaaaaa,
                    bar(
                        bbbbbbbbbbbbbbbbbbbbbbbb,
                        cccccccccccccccccccccc,
                        dddddddddddd,
                        eeeeeeeeeee,
                    ),
                )
    yield
    yield 1
    yield(1, 2)
    yield()
    yield [1, 2]
    yield a: 1
    yield(*a)
    yield *a
    yield(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeeeee
    )
    yield(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeeee
    )
    yield(
        aaaaaaaaaaaaaaaaaaaa,
        bar(
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddd,
            eeeeeeeeeee,
        )
    )
    yield(
        [
            aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddd,
            eeeeeeeeeeeeee,
        ]
    )
end

def m(...)
    bar(...)
    bar(1, ...)
end

def m(*, **, &)
    foo(*, **, &)
end

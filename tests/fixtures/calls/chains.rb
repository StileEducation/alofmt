foo.bar.baz.qux.quux do
    x
    y
end
foo
    .bar(1)
    .baz
    .qux(2)
    .quux do
        x
        y
    end
foo
    .bar
    .baz(1)
    .qux do
        x
        y
    end
foo
    .bar
    .baz do
        x
        y
    end
    .qux
foo
    .bar(1)
    .baz do
        x
        y
    end
foo.bar.baz do
    x
    y
end
foo.bar(1) do
    x
    y
end
foo(1).bar.baz do
    x
    y
end
foo(1)
    .bar(2)
    .baz(3) do
        x
        y
    end
foo.bar[1]
    .baz(1)
    .qux
    .quux do
        x
        y
    end
foo
    .bar { x }
    .baz
    .qux do
        x
        y
    end
foo.bar(1).baz.qux 1 do
    x
    y
end
foo
    .bar do
        x
        y
    end
    .baz
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb(1)
    .cccccccccccccccccccccc(2)
    .dddddddddddddddd
    .eeeeeeeeeeee(3)
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee { x }
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee do
    x
    y
end
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb(1)
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee do
        x
        y
    end
aaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbb.cccccccccccccccccccccc(
    dddddddddddddddd,
    eeeeeeeeeeee,
    ffffffff,
)
aaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbb.cccccccccccccccccccccc.ddddddd(
    dddddddddddddddd,
    eeeeeeeeeeee,
    ffff,
)
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .ddddddd(dddddddddddddddd, eeeeeeeeeeee, ffff)
    .ee
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee.ffff 1
aaaaaaaaaaaaaaaaaaaa
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    &.eeeeeeeeeeee
foo.bar.baz.qux(
    aaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    ccccccccccc,
)
foo
    .bar
    .baz
    .qux(
        aaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccc,
    ) { x }
foo
    .bar
    .baz
    .qux(
        aaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccc,
    ) do
        x
        y
    end
foo
    .bar
    .baz
    .qux(aaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccc)
    .quux
foo
    .bar(1)
    .baz
    .qux(aaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccc)
foo
    .bar(1)
    .baz
    .qux
    .quux(
        aaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccc,
    ) + 1
-foo
    .bar(1)
    .baz
    .qux
    .quux(
        aaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        ccccccccccc,
    )
foo(1)
    .bar(2)
    .baz(3)
    .qux(aaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccc)
foo(aaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccc)
    .bar(2)
    .baz(3)
    .qux
self
    .foo
    .bar(aaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccc)
    .baz(2)
    .qux
@foo
    .bar(aaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccc)
    .baz(2)
    .qux
foo
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc { x }
    .dddddddddddddddd
    .eeeeeeeeeeee
    .fffffffffffffffff
foo
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee
    .fffffffffffffffff.gggggg =
    1
foo
    .bbbbbbbbbbbbbbbbbbbbbbbb
    .cccccccccccccccccccccc
    .dddddddddddddddd
    .eeeeeeeeeeee
    .fffffffffffffffff[
    1
]
foo
    .bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb(1)
    .ccccccccccccccccccccccccccccccc(1)
    .dddddddddddddddddddddddddddddd(1)
foo
    .bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    .ccccccccccccccccccccccccccccccc
    .dddddddddddddddddddddddddddddd
foo.map { x }.select { y }.first
aaaaaaaaaaaaaaaaaaaa
    .map { x }
    .select { y }
    .reject { zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz }
    .first
foo.map { x }.select { y }
foo
    .bar(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        ddd,
    )
    .baz
    .qux { foobar }
foo
    .bar(
        aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        ddd,
    )
    .baz
    .qux do
        foobar
        x
    end
foo.bar aaaaaaaaaaaaaaaaaaaa
                    .bar
                    .baz(
                        bbbbbbbbbbbbbbbbbbbbbbbb,
                        cccccccccccccccccccccc,
                        dddddddddddddddd,
                    )
                    .x

foo { bar }
foo { bar }
foo do
    bar
    baz
end
foo do
    bar
    baz
end
foo {}
foo {}
foo {}
foo {}
foo(1) { x }
foo(1) { x }
foo() { x }
foo() do
    x
    y
end
foo.each { x }
foo.each do
    x
    y
end
foo(bar { x })
foo(bar { x })
foo bar { x }
foo bar do
    x
end
foo bar, baz { x }
foo bar {
            a
            b
        },
        baz
foo bar {
            a
            b
        }.qux
foo a:
            baz {
                a
                b
            }
foo [
            baz {
                a
                b
            },
        ]
foo bar(
            baz do
                a
                b
            end,
        )
expect {
    foo
    bar
}.to raise_error(Foo)
expect do
    foo
    bar
end.to(raise_error(Foo))
expect { foo }.to raise_error(Foo)
foo do
    x
    y
end.bar
foo do
    x
    y
end.bar.baz
foo(1, 2) do
    x
    y
end.baz(3).qux.quux
foo { x }.bar { y }
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    ddddddddddddddd,
) { foobar }
foo(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddd,
) { foobarfoobar }
foo do
    aaaaaaaaaaaaaaaaaaaa(
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeee,
    )
end
foo do
    aaaaaaaaaaaaaaaaaaaa(
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeee,
    )
end
foo do
    # comment
    x
end
foo.each do
    # comment
    x
end
foo do
    x # trailing
end

foo&.bar
foo&.bar(1)
foo&.bar { x }
foo&.bar&.baz
foo.bar&.baz.qux
foo&.()
a&.[](1)
foo
    .bar(1)
    .baz
    &.qux do
        x
        y
    end
foo
    .bar(1)
    &.baz
    .qux do
        x
        y
    end
foo
    &.bar(1)
    .baz
    .qux do
        x
        y
    end
foo.bar&.baz aaaaaaaaaaaaaaaaaaaa,
                          bbbbbbbbbbbbbbbbbbbbbbbb,
                          cccccccccccccccccccccc,
                          dddddddddddddddd

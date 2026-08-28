a.b = 1
a.b.c = 1
foo&.bar = 1
self.foo = 1
foo.bar = baz
foo.bar = [1, 2]
foo.bar = { a: 1 }
foo.bar =
    aaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbb + cccccccccccccccccccccc +
        dddddddddddddddd + eeeee
foo.bar =
    aaaaaaaaaaaaaaaaaaaa(
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeee,
    )
foo.bar = [
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    eeeee,
]
foo.bar = {
    aaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc: dddddddddddddddd,
    e: 1,
}
foo.bar =
    bbbbbbbbbbbbbbbbbbbbbbbb
        .cccccccccccccccccccccc(1)
        .dddddddddddddddd
        .eeeeeeeeeeee
        .fffffffff
        .ggg
foo.bar =
    baz do
        x
        y
    end

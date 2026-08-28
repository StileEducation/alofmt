puts x
foo 1, bar
foo.bar 1, 2
raise Foo, bar
foo a: 1
foo 1, a: 1, b: 2
foo -x
foo !x
puts(-1)
foo *args
foo **opts
foo &blk
foo 1, &blk
foo aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd
foo.bar aaaaaaaaaaaaaaaaaaaa,
                bbbbbbbbbbbbbbbbbbbbbbbb,
                cccccccccccccccccccccc,
                dddddddddddddddd
foo(1).bar aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd
foo.bar.baz.qux aaaaaaaaaaaaaaaaaaaa,
                                bbbbbbbbbbbbbbbbbbbbbbbb,
                                cccccccccccccccccccccc,
                                dddddddddddddddd
foo.bar.baz.qux.quux aaaaaaaaaaaaaaaaaaaa,
                    bbbbbbbbbbbbbbbbbbbbbbbb,
                    cccccccccccccccccccccc,
                    ddddddddddd
self.bar aaaaaaaaaaaaaaaaaaaa,
                  bbbbbbbbbbbbbbbbbbbbbbbb,
                  cccccccccccccccccccccc,
                  dddddddddddddddd
@foo.bar aaaaaaaaaaaaaaaaaaaa,
                  bbbbbbbbbbbbbbbbbbbbbbbb,
                  cccccccccccccccccccccc,
                  dddddddddddddddd
foo[1].bar aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd
foo { x }.bar aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddd
[1, 2].bar aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            ddddddddddddddddddd
foo aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        &blk
foo aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        e: {
            f: 1,
        }
foo bar(
            aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd,
            eeeeee,
        )
foo aaaaaaaaaaaaaaaaaaaa,
        bar(
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd,
            eeeeee,
        )
foo bar(
            aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd,
            eee,
        ),
        1
foo aaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbb + cccccccccccccccccccccc +
            dddddddddddddddd + eeeeeee
foo aaaaaaaaaaaaaaaaaaaa
            .bbbbbbbbbbbbbbbbbbbbbbbb
            .cccccccccccccccccccccc
            .dddddddddddddddd
            .eeeeeeeeeeee
foo [
            aaaaaaaaaaaaaaaaaaaa,
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd,
            eeeeeeeee,
        ]
foo aaaaaaaaaaaaaaaaaaaa: 1,
        bbbbbbbbbbbbbbbbbbbbbbbb: 2,
        cccccccccccccccccccccc: 3,
        dddddddddddddddd: 4
expect(x).to eq(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
)
allow(upload_service).to receive(
    upload_body_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbb,
)
foo.not_to eq(
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    ee,
)
foo.to aaaaaaaaaaaaaaaaaaaa,
bbbbbbbbbbbbbbbbbbbbbbbb,
cccccccccccccccccccccc,
dddddddddddddddd,
eeeeeeeeee
foo.and eq(
                    aaaaaaaaaaaaaaaaaaaa,
                    bbbbbbbbbbbbbbbbbbbbbbbb,
                    cccccccccccccccccccccc,
                    dddddddddddddddd,
                    eeeeee,
                )
foo aaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd do
    x
end
foo 1 do
    x
end
foo 1, 2 do
    x
end
foo.bar 1 do
    x
end
foo(1) { x }
foo {
    x
    y
}.bar 1,
            2
foo do
    x
    y
end.bar 1,
                2
foo do x end.bar 1
foo.bar 1 do
    x
end.baz aaaaaaaaaaaaaaaaaaaa,
bbbbbbbbbbbbbbbbbbbbbbbb,
cccccccccccccccccccccc,
dddddddddddddd

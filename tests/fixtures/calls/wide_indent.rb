foo(
    aaaaaaaaaaaaaaaaaa,
    bar(
        bbbbbbbbbbbbbbbbbbbbb,
        baz(
            cccccccccccccccccccccc,
            qux(
                ddddddddddddddddddddddd,
                eeeeeeeeeeeeeeeeeeeeeeeeee,
                ffffffffffffffffffffffffffff,
            ),
        ),
    ),
)
foo do
    bar do
        baz do
            qux(
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
                bbbbbbbbbbbbbbbbbbbbbbbbbbbb,
                cccccccccccccccccccc,
            )
            qux(
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
                bbbbbbbbbbbbbbbbbbbbbbbbbbbb,
                ccccccccccccccccccccc,
            )
            quux
                .aaaaaaaaaaaaaaaaaaaaaaaaaa
                .bbbbbbbbbbbbbbbbbbbbbbbbbbb
                .cccccccccccccccccccccccccc
                .dd
            {
                aaaaaaaaaaaaaaaaaaaaaaaaaaa: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
                cccccccccccccccccc: dddd,
            }
            aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa +
                bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb + cccccccccccccccc
        end
    end
end
foo.bar do
    baz.qux(
        aaaaaaaaaaaaaaaaaaaaaaaa,
        [
            bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
            ccccccccccccccccccccccccccc,
            ddddddddddddddddddddddddd,
        ],
    )
end

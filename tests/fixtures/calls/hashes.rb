{}
{ a: 1 }
{ a: 1, b: 2 }
{ a => 1 }
{ a: 1, b: { c: 2 } }
{ **opts }
{ a: 1, **opts }
{ a?: 1, b!: 2, _c: 3 }
{ a: 1 }
{
    aaaaaaaaaaaaaaaaaaaa: 1,
    bbbbbbbbbbbbbbbbbbbbbbbb: 2,
    cccccccccccccccccccccc: 3,
    dddddddddddddddd: 4,
}
{
    aaaaaaaaaaaaaaaaaaaa: 1,
    bbbbbbbbbbbbbbbbbbbbbbbb: {
        cccccccccccccccccccccc: 3,
        dddddddddddddddd: 4,
        eeeeeeeeeeeeeeeeeeeeeee: 5,
    },
}
{
    aaaaaaaaaaaaaaaaaaaa: 1,
    bbbbbbbbbbbbbbbbbbbbbbbb: 2,
    cccccccccccccccccccccc: 3,
    ddddddddd: {
        f: 1,
    },
}
{
    aaaaaaaaaaaaaaaaaaaa: 1,
    bbbbbbbbbbbbbbbbbbbbbbbb: 2,
    cccccccccccccccccccccc: 3,
    ddddddddd: [1, 2],
}
{
    aaaaaaaaaaaaaaaaaaaa: 1,
    bbbbbbbbbbbbbbbbbbbbbbbb: 2,
    cccccccccccccccccccccc: 3,
    ddddddddd: foo(1),
}
{
    aaaaaaaaaaaaaaaaaaaa:
        foo(
            bbbbbbbbbbbbbbbbbbbbbbbb,
            cccccccccccccccccccccc,
            dddddddddddddddd,
            eeeeeeeeeeee,
        ),
}
{
    aaaaaaaaaaaaaaaaaaaa:
        bbbbbbbbbbbbbbbbbbbbbbbb + cccccccccccccccccccccc + dddddddddddddddd +
            eeeeeeeeeeee + fff,
}
{
    aaaaaaaaaaaaaaaaaaaa:
        bbbbbbbbbbbbbbbbbbbbbbbb
            .cccccccccccccccccccccc
            .dddddddddddddddd
            .eeeeeeeeeeee
            .fffffffff,
}
{
    aaaaaaaaaaaaaaaaaaaa:
        bbbbbbbbbbbbbbbbbbbbbbbbccccccccccccccccccccccddddddddddddddddeeeeeeeeeeeefffffffffff,
}
{
    aaaaaaaaaaaaaaaaaaaa =>
        bbbbbbbbbbbbbbbbbbbbbbbbccccccccccccccccccccccddddddddddddddddeeeeeeeeeeeefffffffffff,
}
{
    aaaaaaaaaaaaaaaaaaaa: [
        bbbbbbbbbbbbbbbbbbbbbbbb,
        cccccccccccccccccccccc,
        dddddddddddddddd,
        eeeeeeeeeeee,
        ff,
    ],
}
{
    aaaaaaaaaaaaaaaaaaaa: {
        bbbbbbbbbbbbbbbbbbbbbbbb: cccccccccccccccccccccc,
        dddddddddddddddd: eeeeeeeeeeee,
        f: 1,
    },
}
{ a: 1 }.merge(b: 2)
foo.each_with_object({}) { x }
[1, a: 2]
[
    aaaaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbbbbbbbb,
    cccccccccccccccccccccc,
    dddddddddddddddd,
    e: {
        f: 1,
    },
]

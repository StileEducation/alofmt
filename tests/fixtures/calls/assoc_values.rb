foo(a:)
{ a:, b: }
foo(a:, b: 2)
foo(*a, b:)
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: -> { x },
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 1,
}
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: -> do
        x
    end,
}
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: ->(
        x
    ) do
        x
    end,
}
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: <<~EOS,
  x
EOS
}

{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa(
        1,
    ) =>
        nil,
}
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa(
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        100_000_000_000_000_000_000,
        200_000,
    ) => [1, 2],
}
{
    a:
        'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
}
{
    a:
        :aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
}
{
    a:
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.b,
}
{
    a: [
        1,
    ].aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
}
{
    a:
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa do
            x
        end,
}
{
    a:
        foo.bbbbbbbbbbbbbbbbbbbbbbbbbbb(
            1_111_111_111_111_111_111_111,
        ).cccccccccccccccccccccccccccc(2_222_222_222_222_222),
}
{ a: {} }
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: {
    },
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 1,
}
{
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: [],
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 1,
}
foo(
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: {
    },
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: {
    },
)
x =
    foo(
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: 1,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 'aaaaaaaaaaaaa',
    )

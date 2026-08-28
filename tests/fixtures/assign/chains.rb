x = y = z
x = y ||= z
x = @y = self.z = w
x =
    y =
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa +
            bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
x =
    y = [
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    ]
x =
    y = {
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:
            bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    }
x ||=
    y ||=
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa +
            bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
x =
    y =
        z =
            aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa(
                bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
            )
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
    y = [aaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbb]
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx =
    y = aaaaaaaaaaaaaaa(bbbbbbbbbbbbbbbbbbbbb)
foo.bar = baz = 1
foo[1] = baz = 1
x = y = [1, 2].map(&:c)

case foo
in [a, b]
in [*rest]
in [a, *rest, b]
in [a, [b, c]]
in [a]
in [a, b]
in []
in [a, *]
in [*, x, *]
in [*pre, x, *post]
in Foo[a, b]
in Foo[a, b]
in Foo[]
in Foo[]
in Foo::Bar[a, *]
in [a, b] => c
in [a, [b, c] => d]
in [1, [2, 3] => x, *, { y: 1 } | nil]
    1
in ['POST', %r{\A/app/installations/(?<id>[^/]+)/access_tokens\z} => _]
    fake_access_token(::Regexp.last_match[:id])
in [
          'POST',
          %r{\A/repos/(?<o>[^/]+)/(?<r>[^/]+)/stacks/(?<n>\d+)/unstack_something_long\z} =>
              _
      ]
    m
in Foo::Bar::Baz::Qux[
          very_long_key_name_one: String => first_value,
          very_long_key_name_two: Integer
      ]
    m
in [
          very_long_key_name_one,
          very_long_key_name_two,
          very_long_key_name_three,
          *rest
      ] => whole
    m
in [
          very_long_key_name_one,
          very_long_key_name_two,
          very_long_key_name_three,
          *rest
      ]
    foo
in [
          *,
          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
          *
      ]
in Foo[
          *pre,
          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
          *post
      ]
in [
          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
          [
              bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
              ccccccccccccccccccccccccccccccc,
              ddddddddddddddddddddddddd
          ]
      ]
in [
          [b, c],
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
      ]
in [
          b => c,
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
      ]
in [
          AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA =>
              bbbbbbbbbbbbbbbbbbbb
      ]
end

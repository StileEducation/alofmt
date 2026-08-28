case foo
in {}
in a:
in { a:, b: 1, **rest }
in a: { b: [c, *] }
in { a:, b: }
in { a: 1, **nil }
in **nil
in a: 1
in { a: 1, b: 2 }
in a: Integer => i
in a: { b: }
in [{ a: }]
in a: 1
in { a: 1, b: 2 } if x
in Foo[a:]
in Foo[a: 1, b: 2]
in Foo[**nil]
in Foo[{ a: }]
in a: [1, 2]
in a: [b, c] => d
in aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: {
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: Integer,
          cccccccccccccccccccccccccc: String
      }
in aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: Integer
in {
          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: Integer,
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: String,
          **rest
      }
in {
          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: Integer,
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: String,
          **nil
      }
in {
          very_long_key_name_one: String => first_value,
          very_long_key_name_two: Integer => second
      } if some_guard_condition_here?(first_value)
    m
in {
          a: [b, c],
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 1
      }
in {
          a: { b: 1, c: 2 },
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 1
      }
in {
          aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: [
              bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
              ccccccccccccccccccccccccccccccc,
              ddddddddddddddddddddddddd
          ],
          e: 1
      }
in a: {
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb: 1,
          ccccccccccccccccccccccccccccccc: 2,
          dddddddddddddddddddddddddd: 3
      }
in a: Integer =>
          iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii
end

x = 1 if a
x += 1 if a
b = c ? d : e if a
if a
    x = b if c
end
if a
    b =
        begin
            c
        rescue StandardError
            d
        end
end
x = (b ? c : d) if a
x = (b if c) if a
b.c = 2 if a
x, y = 1, 2 if a
x = 1, 2 if a
-> {} if a
b => c if a
-> { c if d } if a
x = (b ? c : d) if a
x = <<~EOS if a
    text
  EOS

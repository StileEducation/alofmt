foo { |a| }
foo { || 1 }
foo { |a, b| a }
foo { it }
foo { _1 + _2 }
foo { |a, (b, c)| }
foo { |(a, b), c| }
foo { |a, ((b, c), d)| }
foo { |(a, *b), c| }
foo { |(a, *), c| }
foo { |a, b; c, d| }
foo { |; a| }
foo { |a,| }
foo { |*| }
foo { |**| }
foo { |&| }
foo { |a, *| }
foo { |*, a| }
foo { |a, b = 1| }
foo { |a, b = 1, *c, d, e:, f: 1, **g, &h| }
foo { |a, **nil| }
foo { |a| }
foo { |a, b; c| x }
foo do |aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccc|
end
foo do |aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb; cccccccccccccccc|
end
foo do |a, b| # comment
end
foo do |a| # trailing
    x
end
foo.each_with_index { |(key, value), index| puts key }
foo.inject(0) { |sum, x| sum + x }
foo.each do |aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccccccccc|
    x
end

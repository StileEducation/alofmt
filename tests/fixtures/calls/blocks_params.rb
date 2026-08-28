foo { |x| x }
foo { |x| x }
foo do |x|
    x
    y
end
foo { |x| }
foo { |x| }
foo { |x; y| x }
foo { |x, (y, z)| x }
foo { |*a, **k, &b| a }
foo { |a, b = 1| a }
foo { || 1 }
foo.each { |a, b| a }
foo(1) { |x| x }
foo 1 do |x|
    x
end
foo bar do |x|
    x
end
foo.each_with_object({}) { |x, h| h }
foo(1, 2) do |aaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbb|
    ccccccccccccccccccccc(dddddddd, eeeeeeeeeeeeee)
end
foo(
    1,
    2,
) do |aaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbb, ccccccccccccccccccccc, dddddddd, eeeeeeeeeeeeeeeeee|
    x
end
foo do |aaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccc, dddddddddddddddd, eeeee|
    x
end
allow(test_socket).to receive(:write) { |thing_to_write|
    written_string << thing_to_write
    orig_test_socket_write.call(thing_to_write)
}
expect(x).to receive(:y).with(1) { |a|
    a
    b
}
foo
    .bar(1)
    .baz
    .qux do |x|
        x
        y
    end
aaaaaaaaaaaaaaaaaaaa
    .map { |x| x }
    .select { |y| y }
    .reject { |z| zzzzzzzzzzzzzzzzzzzzzzzz }
    .first
foo.map { |x| x }.select { |y| y }
foo do |x|
    # comment
    x
end

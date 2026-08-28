foo.each do
    if x.valid?
        process(x)
    elsif x.stale? # too old
        # drop it on the floor
    else
        raise ArgumentError, "unexpected #{x.inspect}"
    end
end
items.map do
    case item
    when String
        item.upcase
    when Integer, Float
        item.to_s
    else
        # unknown, keep as-is
        item
    end
end
foo.each do
    next unless x
    break if x.done?
    x.tick while x.pending?
    begin
        x.run
    rescue Timeout::Error
        retry
    rescue StandardError, ArgumentError
        next
    ensure
        x.close
    end
end
foo.each do
    return x if x.a? && x.b? || x.c?
    return unless x.valid? and x.enabled?
    return x.name.presence || x.default_name || 'anonymous' if x.named?
end
foo(bar) { b if a }
foo(bar) { b if a }
puts((a ? b : c))
puts a ? b : c
foo.select { x.a? ? x.b : x.c }
foo.select { x.a? ? x.b : x.c }
foo.select { x.a? ? x.b : x.c }
b if a
# trailing comment after an if

b if a

d if c
case a
when 1
    b
else # c
end
if a
else # c
end
[1].each { b if a }
loop do
    if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbb
        return foo(bar).baz { x.qux }
    end
    unless aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbbbbbbbbbbbbbbbbb?
        return nil
    end
end
unless aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.include?(bbbbbbbbbbbbbbbbbbbbbbb)
    raise ArgumentError, 'bad'
end
if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa &&
          bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    foo(a, b)
end
unless c
    foo.bar(
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
    )
end

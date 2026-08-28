foo do
    b
rescue StandardError => e
    c
ensure
    d
end
foo.each do |x|
    x.run
rescue A
    retry
else
    done
end
[1].map do |x|
        x
    rescue B
        nil
    end
    .compact

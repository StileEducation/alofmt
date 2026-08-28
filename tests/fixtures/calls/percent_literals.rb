%w[a b]
%w[a b]
['a b', 'c']
['a', 1]
["a\"b", 'c']
['a', 'b\'c']
['a', "#{b}"]
['a', "b\n"]
%i[a b]
[:a, :'b c']
%i[a b?]
['a']
['']
['a', '']
%w[a b#c]
['a', 'b]c']
['a', 'b[c']
%w[a b{c]
['a', "b\\c"]
['a\\b']
%i[a b= [] +]
['a', 'b'.freeze]
x = %w[a b]
foo(%w[a b])
%w[
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    cccccccc
]
['a b', 1]
[:'a b', :c]
['a', :b]
[:a]
['a', "b\tc"]
[:'a b']
[:'a']
%w[é x]
%w[a b]
%i[a A _b b1]
[%w[a], 'b']
[
    'a' \
        'b',
]
[:"a#{b}"]
%w[- +]
%i[- +]
%w[a= b?]
['a', "b\\"]
%w[# b]
%w[#{a} b]
%w[#a b]
%w[a"b c]
%w[b(c d]
[:'a', :b]
[:'a', :b]
[:"a#{b}", :c]
%i[a b].freeze
%w[a b].freeze
['a', 'b', 'c d']
%i[a? b!]
[:'a?', :b]
%w[a b].each { x }
[%w[a b], %w[c d]]
[%w[a b], 1]
['a' * 2, 'b']
['a', "b\#{c}"]
['a\'', 'b']
%w[a b:]
%w[a b,]
%w[a b%]
%w[a b|]
%w[a b]
    .aaa
    .bbb
    .ccc
    .ddd
    .eee
    .fff
    .ggg
    .hhh
    .iii
    .jjj
    .kkk
    .lll
    .mmm
    .nnn
    .ooo
    .ppp
    .qqq
    .rrr
    .sss
    .ttt

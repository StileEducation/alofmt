#!/usr/bin/env ruby
# frozen_string_literal: true

# Scrambles in-style Ruby files with a different formatter, then checks that
# alofmt restores them byte for byte. Each file is classified:
#
#   ok        alofmt(scrambled) == original
#   oracle    the reference itself would not restore it (oracle(scrambled) !=
#             original), so the scramble lost information — not alofmt's bug
#   MISMATCH  the reference restores it and alofmt does not
#   error     alofmt failed on the scrambled source
#
# Usage: roundtrip.rb [--scrambler stree|CMD] [--alofmt PATH] [--report DIR] FILE...
#
# The default scrambler is syntax_tree with its stock options (2-space indent,
# double quotes, no trailing commas). A CMD scrambler is run as `CMD < in > out`.

require 'optparse'
require 'open3'
require 'tmpdir'
require 'syntax_tree'
require 'syntax_tree/plugin/single_quotes'
require 'syntax_tree/plugin/trailing_comma'

abort "syntax_tree 6.2.0 required, have #{SyntaxTree::VERSION}" unless SyntaxTree::VERSION == '6.2.0'

options = { scrambler: 'stree', alofmt: File.expand_path('../target/release/alofmt', __dir__), report: nil }
OptionParser.new do |o|
    o.on('--scrambler S') { |v| options[:scrambler] = v }
    o.on('--alofmt PATH') { |v| options[:alofmt] = v }
    o.on('--report DIR') { |v| options[:report] = v }
end.parse!(ARGV)
files = ARGV
abort 'no files given' if files.empty?

def house(source)
    formatter = SyntaxTree::Formatter.new(source, [], 80, "\n") { |n| ' ' * n * 2 }
    SyntaxTree.parse(source).format(formatter)
    formatter.flush
    formatter.output.join
end

def stock(source)
    # The plugins define constants that flip the defaults; undo them for the
    # scramble by passing explicit options.
    opts = SyntaxTree::Formatter::Options.new(quote: '"', trailing_comma: false)
    formatter = SyntaxTree::Formatter.new(source, [], 80, "\n", options: opts) { |n| ' ' * n }
    SyntaxTree.parse(source).format(formatter)
    formatter.flush
    formatter.output.join
end

def scramble(source, scrambler)
    return stock(source) if scrambler == 'stree'

    out, err, status = Open3.capture3(scrambler, stdin_data: source)
    raise "scrambler failed: #{err}" unless status.success?

    out
end

counts = Hash.new(0)
FileUtils.mkdir_p(options[:report]) if options[:report]
files.each do |path|
    original = File.read(path)
    begin
        scrambled = scramble(original, options[:scrambler])
    rescue StandardError => e
        counts[:unscrambled] += 1
        warn "unscrambled #{path}: #{e.message.lines.first}"
        next
    end
    restored, err, status = Open3.capture3(options[:alofmt], '-', stdin_data: scrambled)
    verdict =
        if !status.success?
            warn "error #{path}: #{err.strip}"
            :error
        elsif restored == original
            :ok
        elsif house(scrambled) != original
            :oracle
        else
            :MISMATCH
        end
    counts[verdict] += 1
    next unless options[:report] && verdict != :ok

    stem = File.join(options[:report], path.tr('/', '_'))
    File.write("#{stem}.scrambled.rb", scrambled)
    File.write("#{stem}.restored.rb", restored) if status.success?
    puts "#{verdict} #{path}"
end
puts counts.sort_by { |k, _| k.to_s }.map { |k, v| "#{k}=#{v}" }.join(' ')
exit(counts[:MISMATCH].zero? && counts[:error].zero? ? 0 : 1)

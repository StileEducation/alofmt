#!/usr/bin/env ruby
# frozen_string_literal: true

# Prints the house-style reference formatting of a Ruby source: syntax_tree
# 6.2.0 with the single-quotes and trailing-comma plugins, 80 columns, and
# the two-units-per-level indent that the prettier plugin drove it with.
# Reads the given file, or stdin when no path is given.

require 'syntax_tree'
require 'syntax_tree/plugin/single_quotes'
require 'syntax_tree/plugin/trailing_comma'

expected = '6.2.0'
abort "syntax_tree #{expected} required, have #{SyntaxTree::VERSION}" unless SyntaxTree::VERSION == expected

source = ARGV.empty? ? $stdin.read : File.read(ARGV.fetch(0))
formatter = SyntaxTree::Formatter.new(source, [], 80, "\n") { |n| ' ' * n * 2 }
SyntaxTree.parse(source).format(formatter)
formatter.flush
print formatter.output.join

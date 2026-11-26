def first_valid = items.filter_map { |item| item if item.valid? }.first

def self.visible_names = items.filter_map { |item| item.name unless item.hidden? }.sort

def generated_class = Class.new { def name = "generated" }.new

def install_method = -> { def answer = 42 }.call

def first_name = items.map { |item| item.name }.first

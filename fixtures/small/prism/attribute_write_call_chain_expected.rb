self.foo = bar
self.foo = Bar.baz

self.foo = Foo.bar.baz.qux.something.else.and_more.really_long.even_more_long.super_duper_long.incredibly_long.more

self.additional_fees_inheritance = Opus::Connect::ProductConfig::SharedModel::Inheritance::TemplateField
  .from_interface(i.additional_fees_inheritance)

self.enabled_inheritance = Opus::Connect::ProductConfig::SharedModel::Inheritance::TemplateField
  .from_interface(i.enabled_inheritance)
self.application_fees_inheritance = Opus::Connect::ProductConfig::SharedModel::Inheritance::TemplateField
  .from_interface(i.application_fees_inheritance)

self&.foo = Bar.baz.qux

class Example
  def from_interface(i)
    self.enabled = i.enabled
    self.additional_fees_inheritance = Opus::Connect::ProductConfig::SharedModel::Inheritance::TemplateField
      .from_interface(i.additional_fees_inheritance)
  end
end

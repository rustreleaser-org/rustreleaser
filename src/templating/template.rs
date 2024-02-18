pub const TEMPLATE: &str = r#"
# typed: false
# frozen_string_literal: true

class {{ formula }} < Formula
  desc "{{ description }}"
  homepage "{{ homepage }}"
  version "{{ version }}"
  {{#if license }}
  license "{{ license }}"
  {{/if}}
  {{#if macos }}
{{lines 1}}
  on_macos do
    if Hardware::CPU.intel?
    {{#with macos}}
      url "{{ url }}"
      sha256 "{{ hash }}"
     {{/with}}
    end
  end
  {{/if}}
  {{#if linux }}
{{lines 1}}
  on_linux do
    if Hardware::CPU.intel?
    {{#with linux}}
      url "{{ url }}"
      sha256 "{{ hash }}"
    {{/with}}
    end
  end
  {{/if}}
{{#if (or linux macos)}}
{{lines 1}}
  def install
    bin.install Dir["*"]
  end
{{lines 1}}
{{/if}}
end
"#;

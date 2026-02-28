require "minitest/autorun"
require_relative "../hello"

class TestHello < Minitest::Test
  def test_greet
    assert_equal "hello", greet
  end
end

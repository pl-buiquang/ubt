package main

import "testing"

func TestMain(t *testing.T) {
	// trivial test
	if 1+1 != 2 {
		t.Fatal("math is broken")
	}
}

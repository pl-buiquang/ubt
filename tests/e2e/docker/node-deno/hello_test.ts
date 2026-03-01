Deno.test("basic math", () => {
  if (1 + 1 !== 2) {
    throw new Error("math is broken");
  }
});

<!-- Vendored from payment (.claude/skills/php-writing-tests/references/exception-assertion.md) on 2026-11-18. Adapted to this project: renamed from exception-assertion.md, the try/catch versus expectException pair became a conditional or substring assertion versus matches! on the error variant, worked from tests/application/handlers/. -->

# Error assertion shape

Use this shape when asserting on an answered error, preferring `matches!` on the variant over a conditional or a
substring of the message.

```rust
// Asserting on an error. Match the variant with assert!(matches!(...)), which names the expected
// error in the test and fails with the actual value. Assert the message only when the message is
// the contract (an endpoint body carries it), and then compare it whole with assert_eq!, never
// with contains. A match or an if let around the result is a conditional in the test body, which
// the testing rules forbid.

// Prohibited: a conditional to reach the error, and a substring of its message
if let Err(failure) = tax {
    assert!(failure.to_string().contains("tax rules"));
}

// Correct: the variant, matched in one assertion
assert!(matches!(tax, Err(CalculationFailure::TaxRulesNotFound(_))));

// Correct, when the message is the contract: the whole message, compared with assert_eq!
assert_eq!(body, json!({"code": expected_code, "message": message}));
```

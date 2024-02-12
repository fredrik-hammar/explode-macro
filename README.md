Bringing back the venerable LISP explode[^1] function from the past!

[^1]: See [MACLISP EXPLODE Function](https://www.maclisp.info/pitmanual/charac.html#EXPLODE)

# Example

```rust
use explode::explode;

let expected: [char; 5] = ['h', 'e', 'l', 'l', 'o'];
assert_eq!(expected, explode!(hello));
assert_eq!(expected, explode!("hello"));

let expected: [u8; 5] = [b'h', b'e', b'l', b'l', b'o'];
assert_eq!(expected, explode!(b"hello"));
```

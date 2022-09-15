# Search design ideas

Ultimately, we want searching to be more simple and intuitive than in Cheat
Engine.

## Goals

*   Require fewer mouse clicks to perform common operations.
*   Allow for basic scan automation, e.g. "monotonically increasing value"

## Proposal: Textual Search

Create a textual query language that can be used to scan memory.

**Types:**

| Type | Example | Notes |
| ---- | ------- | ----- |
| Integral | `625` | Can match ints and floats
| Float | `625.6e7` | Matches only floats
| Byte strings | `b(FF 66 52 ??)` | Supports wildcard bytes

**Comparators:**

| Type | Example | Notes |
| ---- | ------- | ----- |
| Equivalence | `652` (or `=652`) | Floats compare within $\epsilon$
| Comparator | `>=625` |
| Within range | `0 .. 5.3e5` |
| Is valid pointer | `is ptr` |
| Is  | `bytes:4` |
| Alignment | `align:4` |

**Basic math:**

| Type | Example | Notes |
| ---- | ------- | ----- |
| Addition | `0xFFFF71E0 + 4` | Same as searching `0xFFFF71E4`
| Subtraction | `0xFFFF71E0 - 4` |

**Special tokens:**

| Type | Example | Notes |
| ---- | ------- | ----- |
| Previous value | `prev` | The previous search value at that address

**Pointer scans (expensive):**

| Type | Example | Notes |
| ---- | ------- | ----- |
| Direct reference | `&(152)` | "Pointer to an int 152"
| Offset reference | `&(132.2) - 16` | "Pointer to a struct/array where<br> the 16th byte is a float 132.2"

**Combinators** can combine simple searches:

| Type | Example | Notes |
| ---- | ------- | ----- |
| And | `>0xEE00 and b(* * * 62)` |
| Or | `b(ff 82 46 21) or b(ff 72 48 21)` |

**Repeating searches** continue until stopped or until no values remain:

| Type | Example | Notes |
| ---- | ------- | ----- |
| Constant | `always (625..627)` |
| Keeps increasing | `always >= prev` |
| Keeps increasing by 2 | `always (== prev or == prev + 2)` |
| Doesn't change | `always == prev` |

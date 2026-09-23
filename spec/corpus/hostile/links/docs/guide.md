# Links

Good: [page](other.md), [anchor](other.md#a-section), [self](#links),
[directory](sub/), [directory without slash](sub), [root-relative](/README.md#install),
[out of docs](../src/main.rs#L1), [query](other.md?plain=1#a-section),
[spaces](<a file.md>), [percent](a%20file.md), [title](other.md "Other"), [padded](  other.md  ).

External, never checked: [web](https://example.com/nope), [mail](mailto:a@example.com),
[scheme-relative](//example.com/x), [custom](cairn:72).

Broken: [missing](missing.md), [anchor](other.md#no-such-section),
[self anchor](#no-such-heading), [case](OTHER.md), [readme anchor](/README.md#nowhere),
[escapes](../../outside.md), ![image](missing.png).

Empty fragment is no fragment: [empty](other.md#). Empty destination is not a link: [nothing]().

Not links: `[code span](missing.md)`, ``[double `span`](missing.md)``, \[escaped](missing.md) is not one either,
[unclosed](missing.md, [broken link text(missing.md).

[![nested image](missing-image.png)](other.md)

[ref]: missing-ref.md
[good ref]: <other.md#a-section> "title"

```
[fenced](missing.md)
```

~~~markdown
[tilde fenced](missing.md)
~~~

    [indented code](missing.md)

- a list item
    [continuation, not code](other.md)

## Heading with a [link](missing-in-heading.md)

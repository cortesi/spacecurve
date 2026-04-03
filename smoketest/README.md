# spacecurve smoketests

This directory holds the first `eguidev` Luau smoketests for the native
`scurve` GUI.

Each script is self-contained:

- it selects its own starting state with `fixture(...)`
- it drives only explicit widget ids
- it asserts visible behaviour instead of internal implementation details

Run the suite with:

```sh
edev smoke
```

# Inlined Pyramid Fixtures (Dir and File)

## Naming
Directories are named by letters of the alphabet, in order of top-down and
left-right. Lowercase are Included values and uppercase are Actual values.

Files are named numerically, unique to their parent Dir, in order. A prefix of
0 indicates an Actual value and anything else indicates Included.

Each directory has at least one actual File. 

## Categories
- Valid: Should return successfully
- Invalid: Should throw error

### Example Variations
Not all of these are provided as fixtures.

#### Valid: abc
```
  a
 / \
b   c
```

### Valid: abcdefg
```
      a
    /   \
   /     \
  b       c
 / \     / \
d   e   f   g
```


### Valid: abcdefg_ef
```
      a
    /   \
   /     \
  b       c
 / \     / \
d   e+--f   g
```

### Valid: abcdefg_ec
```
      a
    /   \
   /     \
  b  +----c
 / \ |   / \
d   e+  f   g
```

### Invalid: abcdefg_ec_ge
```
      a
    /   \
   /     \
  b  +----c
 / \ |   / \
d   e+  f   g
    |_______+

```




# Collection Functions

## `std.push(list, value)`

Add a value to the end of a list.

```
let items = [1, 2, 3]
std.push(items, 4)  # items is now [1, 2, 3, 4]
```

## `std.pop(list)`

Remove and return the last value from a list.

```
let items = [1, 2, 3]
let last = std.pop(items)  # last = 3, items = [1, 2]
```

## `std.first(list)`

Get the first element of a list.

```
std.first([1, 2, 3])  # 1
```

## `std.last(list)`

Get the last element of a list.

```
std.last([1, 2, 3])  # 3
```

## `std.includes(list, value)`

Check if a list contains a value.

```
std.includes([1, 2, 3], 2)  # true
```

## `std.chunk(list, size)`

Split a list into chunks of the given size.

```
std.chunk([1, 2, 3, 4, 5], 2)  # [[1, 2], [3, 4], [5]]
```

## `std.filter(list, predicate)`

Filter a list by a predicate function.

```
let numbers = [1, 2, 3, 4, 5]
let evens = std.filter(numbers, is_even)  # [2, 4]
```

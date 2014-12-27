puresugar
=========

This preprocessor implements what is declined [here](https://github.com/purescript/purescript/issues/777).
Please think twice before using this in libraries because it will force users to install this preprocessor and to integrate it in their build.
I think complicated nested array and object literals are mostly found in applied end-product code where you have more freedom in using things like this without causing discomfort to anyone.

### Sugar examples
a)
```haskell
data Props = Props #
  greeting :: String
```
desugars to
```haskell
data Props = Props 
  { greeting :: String }
```
b)
```haskell
initialState = State # name: ""
```
desugars to
```haskell
initialState = State { name: "" }
```
c)
```haskell
combo = run @
  quux @ (foo # x: 1, y : 2), (bar # x: 1, y: 1) #
    a : 123
    b : 45678
  bazz # huh: "this stuff works!"
```
desugars to
```haskell
combo = run 
  [ quux [ (foo { x: 1, y : 2 }), (bar { x: 1, y: 1 }) ]
    { a : 123
    , b : 45678 }
  , bazz { huh: "this stuff works!" } ]
```
Check test_input and test_output files.

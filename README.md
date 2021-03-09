# tron-solc

V8 wrapper for solc compiler.

## Version

Tron-Solidity v5.15

## Usage

Use branch: https://github.com/OpenZeppelin/openzeppelin-contracts/tree/release-v2.5.0

```js
pragma solidity >=0.5.0 <0.6.0;

import "@openzeppelin/contracts/ownership/Ownable.sol";
// or
import "https://raw.githubusercontent.com/OpenZeppelin/openzeppelin-contracts/release-v2.5.0/contracts/ownership/Ownable.sol"

// ...
```

## Example

```rust
use solc::{Compiler, Input};

fn main() {
    let code = r#"
    import "@openzeppelin/contracts/ownership/Ownable.sol";

    contract Store is Ownable {
        uint256 internal value;

        function reset() public {
            value = 0;
        }

        function setValue(uint256 v) public {
            value = v;
        }
    }
    "#;
    let input = Input::new().optimizer(0).source("Store.sol", code.into());
    let output = Compiler::new().unwrap().compile(input).unwrap();

    if output.has_errors() {
        output.format_errors();
    }

    println!("{}", output.pretty_abi_of("Store").unwrap());
    println!("{}", output.bytecode_of("Store").unwrap());
}
```

// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract DanteToken is ERC20, Ownable {
  constructor(string memory name_, string memory symbol_) ERC20(name_, symbol_) {
  }

  function burn(address account, uint amount) public onlyOwner {
    _burn(account, amount);
  }

  function mint(address account, uint amount) public onlyOwner {
    _mint(account, amount);
  }
}
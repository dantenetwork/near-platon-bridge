// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@hthuang/contracts/ContractBase.sol";
import "./DanteToken.sol";

contract Locker is ContractBase {
    // Destination contract info
    struct DestnContract {
        string contractAddress; // destination contract address
        string funcName; // destination contract action name
        bool used;
    }

    // Dante token address
    DanteToken public token;

    // Cross-chain destination contract map
    mapping(string => mapping(string => DestnContract)) public destnContractMap;

    // Cross-chain permitted contract map
    mapping(string => mapping(bytes4 => string)) public permittedContractMap;

    function setTokenContract(address _address) external onlyOwner {
        token = DanteToken(_address);
    }

    function get_address(uint8[] memory u) internal pure returns (address) {
        uint160 y = 0;
        for (uint i = 0; i < u.length; i++) {
            y <<= 8;
            y += u[i];
        }
        return address(y);
    }

    /**
     * Map tokens to another chain
     * @param _toChain - to chain name
     * @param _num - number of tokens that will be sent to chain `_toChain`
     */
    function transferToken(
        string calldata _toChain,
        string calldata _toAddress,
        uint128 _num
    ) external {
        // Burn tokens
        token.burn(msg.sender, _num);

        // Construct payload
        Payload memory data;
        data.items = new PayloadItem[](2);
        PayloadItem memory item = data.items[0];
        item.name = "to";
        item.msgType = MsgType.EvmString;
        item.value = abi.encode(_toAddress);
        item = data.items[1];
        item.name = "num";
        item.msgType = MsgType.EvmU128;
        item.value = abi.encode(_num);

        mapping(string => DestnContract) storage map = destnContractMap[_toChain];
        DestnContract storage destnContract = map["receiveToken"];
        require(destnContract.used, "action not registered");

        // Construct message
        ISentMessage memory message;
        message.toChain = _toChain;
        message.session = Session(0, "");
        message.content = Content(destnContract.contractAddress, destnContract.funcName, data);

        crossChainContract.sendMessage(message);
    }

    /**
     * Receive greeting info from other chains
     * @param _payload - payload which contains greeting message
     */
    function receiveToken(Payload calldata _payload) public {
        require(
            msg.sender == address(crossChainContract),
            "Locker: caller is not CrossChain"
        );

        // Parse payload
        (uint8[] memory _u8arry) = abi.decode(_payload.items[0].value, (uint8[]));
        (uint128 _num) = abi.decode(_payload.items[1].value, (uint128));
        address _to = get_address(_u8arry);
        
        // Mint tokens
        token.mint(_to, _num);
    }

    ///////////////////////////////////////////////
    /////    Send messages to other chains   //////
    ///////////////////////////////////////////////

    /**
     * Register destination contract info
     * @param _funcName - function name to be called
     * @param _toChain - destination chain name
     * @param _contractAddress - destination contract address
     * @param _contractFuncName - contract function name
     */
    function registerDestnContract(
        string calldata _funcName,
        string calldata _toChain,
        string calldata _contractAddress,
        string calldata _contractFuncName
    ) external onlyOwner {
        mapping(string => DestnContract) storage map = destnContractMap[_toChain];
        DestnContract storage destnContract = map[_funcName];
        destnContract.contractAddress = _contractAddress;
        destnContract.funcName = _contractFuncName;
        destnContract.used = true;
    }

    ///////////////////////////////////////////////
    ///    Receive messages from other chains   ///
    ///////////////////////////////////////////////

    /**
     * Authorize contracts of other chains to call the functions of this contract
     * @param _chainName - from chain name
     * @param _sender - sender of cross chain message
     * @param _funcName - action name which allowed to be invoked
     */
    function registerPermittedContract(
        string calldata _chainName,
        string calldata _sender,
        bytes4 _funcName
    ) external onlyOwner {
        mapping(bytes4 => string) storage map = permittedContractMap[
            _chainName
        ];
        map[_funcName] = _sender;
    }

    /**
     * This verify method will be invoked by the CrossChain contract automatically, ensure that only registered contract(registerSourceContract) calls are allowed
     * @param _chainName - chain name of cross chain message
     * @param _funcName - contract action name of cross chain message
     * @param _sender - cross chain message sender
     */
    //  Will be deprecated soon
    function verify(
        string calldata _chainName,
        bytes4 _funcName,
        string calldata _sender
    ) public view virtual returns (bool) {
        mapping(bytes4 => string) storage map = permittedContractMap[
            _chainName
        ];
        string storage sender = map[_funcName];
        require(
            keccak256(bytes(sender)) == keccak256(bytes(_sender)),
            "Sender does not match"
        );
        return true;
    }
}

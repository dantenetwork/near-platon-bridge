const debug = require('debug')('ck');
const BN = require('bn.js');

const CrossChain = artifacts.require('@hthuang/contracts/TwoPhaseCommitCrossChain');
const MessageVerify = artifacts.require('@hthuang/contracts/MessageVerify');
const Locker = artifacts.require("Locker");
const DanteToken = artifacts.require("DanteToken");

const eq = assert.equal.bind(assert);
const SEND_TOKEN_NUM = '100';

contract('Locker', function(accounts) {
    let owner = accounts[0];
    let user1 = accounts[1];

    let crossChain;
    let locker;
    let token;
    let messageVerify;
    
    let initContract = async function() {
        crossChain = await CrossChain.new('PLATONEVMDEV');
        messageVerify = await MessageVerify.new();
        locker = await Locker.deployed();
        token = await DanteToken.deployed();
        await locker.setTokenContract(DanteToken.address);
        // Mint token
        await token.mint(user1, '100');

        // Transfer ownership to locker
        await token.transferOwnership(locker.address);

        await crossChain.setVerifyContract(messageVerify.address);

        // register cross-chain contract address
        await locker.setCrossChainContract(crossChain.address);

        // register porters
        await crossChain.changePortersAndRequirement([user1], 1);

        // register target
        await locker.registerDestnContract('receiveToken', 'NEAR', 'contract_address', 'receive_token');
    }

    before(async function() {
        await initContract();
    });

    describe('Transfer Token', function() {
        it('should execute successfully ', async () => {
            await locker.transferToken('NEAR', 'near_address', SEND_TOKEN_NUM, {from: user1});
        });
    });

    describe('Receive Token', function() {
        it('should execute successfully', async () => {
            let to = Locker.address;
            let action = '0x396f7242';
            let item1 = {
                name: 'to',
                msgType: 12,
                value: '0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000f100000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000061000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000d200000000000000000000000000000000000000000000000000000000000000d70000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000008c0000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000008600000000000000000000000000000000000000000000000000000000000000d100000000000000000000000000000000000000000000000000000000000000c800000000000000000000000000000000000000000000000000000000000000ae00000000000000000000000000000000000000000000000000000000000000c5000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000d9000000000000000000000000000000000000000000000000000000000000009e000000000000000000000000000000000000000000000000000000000000009f0000000000000000000000000000000000000000000000000000000000000014',
            };
            let item2 = {
                name: 'num',
                msgType: 5,
                value: '0x0000000000000000000000000000000000000000000000000000000000000100',
            };
            let calldata = {items: [item1, item2]};
            let argument = [1, 'NEAR', 'near_sender', 'near_signer', [], to, action, calldata, [0, '0x'], 0];
            await crossChain.receiveMessage(argument, {from: user1});
            await crossChain.executeMessage('NEAR', 1);
            let context = await crossChain.getCurrentMessage();
            assert(context.id == '1');
            let b = await token.balanceOf('0xC0F1706106D2d7208C6586d1C8Aec520d99E9F14');
            assert(b == '256');
        });
    });
});
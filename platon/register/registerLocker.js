const Web3 = require('web3');
const fs = require('fs');
const ethereum = require('./ethereum');
const { program } = require('commander');
const config = require('config');

let web3;
let netConfig;
let contract;
let tokenContract;

const ABI_PATH = './build/contracts/Locker.json';
const TOKEN_ABI_PATH = './build/contracts/DanteToken.json';
const CONTRACT_KEY_NAME = 'lockerContractAddress';
const TOKEN_CONTRACT_KEY_NAME = 'tokenContractAddress';
const METHOD_KEY_NAME = 'receiveToken';

// Private key
let testAccountPrivateKey = fs.readFileSync('./.secret').toString();

// Get current date
function getCurrentDate() {
  var today = new Date();
  return today.toString();
}

function init(chainName) {
    netConfig = config.get(chainName);
    if (!netConfig) {
        console.log('Config of chain (' + chainName + ') not exists');
        return false;
    }

    // Load contract abi, and init contract object
    const contractRawData = fs.readFileSync(ABI_PATH);
    const contractAbi = JSON.parse(contractRawData).abi;
    
    const tokenContractRawData = fs.readFileSync(TOKEN_ABI_PATH);
    const tokenContractAbi = JSON.parse(tokenContractRawData).abi;

    web3 = new Web3(netConfig.nodeAddress);
    web3.eth.handleRevert = true;
    contract = new web3.eth.Contract(contractAbi, netConfig[CONTRACT_KEY_NAME]);
    tokenContract = new web3.eth.Contract(tokenContractAbi, netConfig[TOKEN_CONTRACT_KEY_NAME]);

    return true;
}

async function initialize() {
  // Set cross chain contract address
  await ethereum.sendTransaction(web3, netConfig.chainId, contract, 'setCrossChainContract', testAccountPrivateKey, [netConfig.crossChainContractAddress]);
  // Set token contract
  await ethereum.sendTransaction(web3, netConfig.chainId, contract, 'setTokenContract', testAccountPrivateKey, [netConfig[TOKEN_CONTRACT_KEY_NAME]]);
}

async function registerDestnContract(chainName) {
  let destConfig = config.get(chainName);
  if (!destConfig) {
      console.log('Config of dest chain (' + chainName + ') not exists');
      return false;
  }

  const interface = JSON.parse(fs.readFileSync('./config/interface.json'));
  if (!interface[destConfig.interface]) {
    console.log('Interface of dest chain (' + chainName + ') not exists');
    return false;
  }

  // Register contract info for sending messages to other chains
  await ethereum.sendTransaction(web3, netConfig.chainId, contract, 'registerDestnContract', testAccountPrivateKey,
    [METHOD_KEY_NAME, chainName, destConfig[CONTRACT_KEY_NAME], interface[destConfig.interface][METHOD_KEY_NAME]]);

  await ethereum.sendTransaction(web3, netConfig.chainId, contract, 'registerPermittedContract', testAccountPrivateKey,
    [chainName, destConfig[CONTRACT_KEY_NAME], interface[netConfig.interface][METHOD_KEY_NAME]]);
}

async function transferToken(toChain, to, num) {
  let addr = {};
  let destConfig = config.get(toChain);
  if (destConfig.interface == 'EVM') {
    addr.chainType = 1;
    addr.evmAddress = to;
    addr.otherAddress = '';
  }
  else if (destConfig.interface == 'INK') {
    addr.chainType = 2;
    addr.evmAddress = '0x0000000000000000000000000000000000000000';
    addr.otherAddress = to;
  }
  else if (destConfig.interface == 'NEAR') {
    addr.chainType = 3;
    addr.evmAddress = '0x0000000000000000000000000000000000000000';
    addr.otherAddress = to;
  }
  await ethereum.sendTransaction(web3, netConfig.chainId, contract, 'transferToken', testAccountPrivateKey,
    [toChain, addr, num]);
}

async function getTokenAmount(account) {
  return await ethereum.contractCall(tokenContract, 'balanceOf', [account]);
}

async function transfer(address) {
  await ethereum.sendTransaction(web3, netConfig.chainId, tokenContract, 'transferOwnership', testAccountPrivateKey, [address]);
}

async function mint(address, num) {
  await ethereum.sendTransaction(web3, netConfig.chainId, tokenContract, 'mint', testAccountPrivateKey, [address, num]);
}

(async function () {
  function list(val) {
    return val.split(',')
  }

  program
      .version('0.1.0')
      .option('-i, --initialize <chain name>', 'Initialize locker contract')
      .option('-r, --register <chain name>,<dest chain name>', 'Register destination chain contract', list)
      .option('-s, --send <chain name>,<dest chain name>,<account id>,<token number>', 'Send transfer token message', list)
      .option('-g, --get <chain name>,<account id>', 'Get token amount', list)
      .option('-t, --transfer <chain name>,<address>', 'Transfer ownership', list)
      .option('-m, --mint <chain name>,<address>,<token number>', 'Mint token', list)
      .parse(process.argv);

  if (program.opts().initialize) {
      if (!init(program.opts().initialize)) {
          return;
      }
      await initialize();
  }
  else if (program.opts().register) {
      if (program.opts().register.length != 2) {
          console.log('2 arguments are needed, but ' + program.opts().register.length + ' provided');
          return;
      }
      
      if (!init(program.opts().register[0])) {
          return;
      }
      await registerDestnContract(program.opts().register[1]);
  }
  else if (program.opts().send) {
    if (program.opts().send.length != 4) {
        console.log('4 arguments are needed, but ' + program.opts().send.length + ' provided');
        return;
    }

    if (!init(program.opts().send[0])) {
        return;
    }
    await transferToken(program.opts().send[1], program.opts().send[2], program.opts().send[3]);
  }
  else if (program.opts().get) {
    if (program.opts().get.length != 2) {
        console.log('2 arguments are needed, but ' + program.opts().get.length + ' provided');
        return;
    }

    if (!init(program.opts().get[0])) {
        return;
    }
    let greeting = await getTokenAmount(program.opts().get[1]);
    console.log('greeting', greeting);
  }
  else if (program.opts().transfer) {
      if (program.opts().transfer.length != 2) {
          console.log('2 arguments are needed, but ' + program.opts().transfer.length + ' provided');
          return;
      }
      
      if (!init(program.opts().transfer[0])) {
          return;
      }
      await transfer(program.opts().transfer[1]);
  }
  else if (program.opts().mint) {
      if (program.opts().mint.length != 3) {
          console.log('3 arguments are needed, but ' + program.opts().transfer.length + ' provided');
          return;
      }
      
      if (!init(program.opts().mint[0])) {
          return;
      }
      await mint(program.opts().mint[1], program.opts().mint[2]);
  }
}());
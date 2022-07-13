const nearAPI = require('near-api-js');
const path = require('path');
const homedir = require('os').homedir();
const credentialsPath = path.join(homedir, '.near-credentials');
const networkId = 'testnet';
const lockerContractId =
  'ae997e9b674480cb9bc88765242c3cbf71267044b51a2b8f99f20028aa441d89';

const nodeUrl = `https://rpc.${networkId}.near.org`;
const gas = 30000000000000;

const nearConfig = {
  networkId,
  keyStore: new nearAPI.keyStores.UnencryptedFileSystemKeyStore(
    credentialsPath
  ),
  nodeUrl,
};

// destination contract information
const Chains = [
  // Platon
  {
    destinationContract: '0xC0F1706106D2d7208C6586d1C8Aec520d99E9F14',
    destinationActionName: '0x396f7242',
    destinationChainName: 'PlatONTEST',
  }
];

(async function init() {
  for (let i in Chains) {
    // Chains.forEach(async (chain) => {
    const near = await nearAPI.connect(nearConfig);
    let account = await near.account(lockerContractId);

    // Register contract info for sending messages to other chains
    await account.functionCall({
      contractId: lockerContractId,
      methodName: 'register_dst_contract',
      args: {
        action_name: 'transfer_token',
        chain_name: Chains[i].destinationChainName,
        contract_address: Chains[i].destinationContract,
        contract_action_name: Chains[i].destinationActionName,
      },
      gas,
    });

    await account.functionCall({
      contractId: lockerContractId,
      methodName: 'register_permitted_contract',
      args: {
        chain_name: Chains[i].destinationChainName,
        sender: Chains[i].destinationContract,
        action_name: 'ft_mint',
      },
      gas,
    });
  }
})();

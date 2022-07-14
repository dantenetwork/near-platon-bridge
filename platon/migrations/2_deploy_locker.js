const Locker = artifacts.require("Locker");
const DanteToken = artifacts.require("DanteToken");
const fs = require("fs");

module.exports = async function (deployer, network) {
    await deployer.deploy(DanteToken, "DANTE", "DAT");
    await deployer.deploy(Locker);

    // Update config
    if (network.indexOf('-fork') != -1) {
        return;
    }
    
    const contractAddressFile = './config/default.json';
    let data = fs.readFileSync(contractAddressFile, 'utf8');
    let jsonData = JSON.parse(data);
    if (!jsonData[network]) {
        console.warn('There is no config for: ', network, ', please add.');
        jsonData[network] = {};
    }

    jsonData[network].lockerContractAddress = Locker.address;
    jsonData[network].tokenContractAddress = DanteToken.address;
    fs.writeFileSync(contractAddressFile, JSON.stringify(jsonData, null, '\t'));
};
const Locker = artifacts.require("Locker");
const DanteToken = artifacts.require("DanteToken");

module.exports = async function (deployer) {
    await deployer.deploy(DanteToken, "DANTE", "DAT");
    await deployer.deploy(Locker);
};
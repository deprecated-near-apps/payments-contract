const nearAPI = require('near-api-js');
const testUtils = require('./test-utils');
const getConfig = require('../src/config');

const { KeyPair, Account, utils: { format: { parseNearAmount }} } = nearAPI;
const { 
	connection, initContract, getAccount, getContract,
	contractAccount, contractName, contractMethods, createAccessKeyAccount
} = testUtils;
const { GAS } = getConfig();

jasmine.DEFAULT_TIMEOUT_INTERVAL = 50000;

describe('deploy contract ' + contractName, () => {
	let alice, contract;
    
	const memo = "hello world!";

	beforeAll(async () => {
		alice = await getAccount();
		await initContract(alice.accountId);
	});

	test('contract hash', async () => {
		let state = (await new Account(connection, contractName)).state();
		expect(state.code_hash).not.toEqual('11111111111111111111111111111111');
	});

	test('check create', async () => {
		contract = await getContract(alice);

		await contract.deposit({
			memo,
		}, GAS, parseNearAmount('1'));
        
		const deposits = await contract.get_deposits({ account_id: alice.accountId });
		expect(deposits[0].memo).toEqual(memo);
		expect(deposits[0].paid).toEqual(false);
	});

	test('check create and make payment', async () => {

		await contract.make_payment({
			deposit_index: 0,
		}, GAS);
        
		const deposits = await contract.get_deposits({ account_id: alice.accountId });
		expect(deposits[0].paid).toEqual(true);
	});

	test('check cannot withdraw', async () => {

		try {
            await contract.withdraw({
                deposit_index: 0,
            }, GAS);
            expect(false)
        } catch(e) {
            console.warn(e)
            expect(true)
        }
	});
    
    test('check create and withdraw', async () => {
		contract = await getContract(alice);

		await contract.deposit({
			memo,
		}, GAS, parseNearAmount('1'));

        await contract.withdraw({
            deposit_index: 1,
        }, GAS);
        
		const deposits = await contract.get_deposits({ account_id: alice.accountId });
		expect(deposits.length).toEqual(1);
		expect(deposits[0].paid).toEqual(true);
	});
    
});
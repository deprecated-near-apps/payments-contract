import React, { useEffect, useState } from 'react';
import * as nearAPI from 'near-api-js';
import { GAS, parseNearAmount } from '../state/near';
import {
    createAccessKeyAccount,
    getContract,
} from '../utils/near-utils';

const {
    KeyPair,
    utils: { format: { formatNearAmount } }
} = nearAPI;

export const Contract = ({ near, update, account }) => {
    if (!account) return null;

    const [memo, setMemo] = useState('');
    const [deposits, setDeposits] = useState([]);
    const [amount, setAmount] = useState('');

    useEffect(() => {
        loadDeposits();
    }, []);

    const loadDeposits = async () => {
        const contract = getContract(account);
        setDeposits(await contract.get_deposits({ account_id: account.accountId }))
    };

    const handleDeposit = async () => {
        const contract = getContract(account);
        await contract.deposit({
            memo
        }, GAS, parseNearAmount(amount))
        loadDeposits()
    };

    const handlePayment = async (deposit_index) => {
        const contract = getContract(account);
        await contract.make_payment({
            deposit_index
        }, GAS)
        loadDeposits()
    };

    const handleWithdraw = async (deposit_index) => {
        const contract = getContract(account);
        try {
            await contract.withdraw({
                deposit_index
            }, GAS)
        } catch(e) {
            alert('payment already confirmed')
        }
        loadDeposits()
    };

    return <>
        <h3>Make a Deposit for the Tournament</h3>
        <p>Tournament X: 5</p>
        <p>Tournament Y: 10</p>
        <input placeholder="Memo" value={memo} onChange={(e) => setMemo(e.target.value)} />
        <input placeholder="Amount (N)" value={amount} onChange={(e) => setAmount(e.target.value)} />
        <br />
        <button onClick={() => handleDeposit()}>Handle Deposit</button>

        {
            deposits.map(({ memo, paid, amount }, i) => <>
                <p>
                    {memo} - {formatNearAmount(amount, 2)} - {paid ? 'true' : 'false'}
                    <br />
                    <button onClick={() => handlePayment(i)}>Handle Payment</button>
                    <button onClick={() => handleWithdraw(i)}>Handle Withdraw</button>
                </p>
            </>)
        }

    </>;
};


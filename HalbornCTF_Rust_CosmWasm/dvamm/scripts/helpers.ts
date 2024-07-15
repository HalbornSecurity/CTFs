import 'dotenv/config'
import {
    Coins,
    isTxError,
    LCDClient,
    MnemonicKey,
    Msg,
    MsgExecuteContract,
    MsgInstantiateContract,
    MsgMigrateContract,
    MsgStoreCode,
    Tx,
    Wallet
} from '@terra-money/feather.js';
import {
    readFileSync,
    writeFileSync,
} from 'fs'
import path from 'path'
import { CustomError } from 'ts-custom-error'

export const ARTIFACTS_PATH = '../artifacts'

export function readArtifact(name: string = 'artifact') {
    try {
        const data = readFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), 'utf8')
        return JSON.parse(data)
    } catch (e) {
        return {}
    }
}

export interface Client {
    wallet: Wallet
    terra: LCDClient
}

export function newClient(): Client {
    const client = <Client>{}
    const lcd = new LCDClient({
        // key must be the chainID
        'localterra': {
          lcd: 'http://localhost:1317',
          chainID: 'localterra',
          gasAdjustment: 1.75,
          gasPrices: { uluna: 0.015 },
          prefix: 'terra', // bech32 prefix, used by the LCD to understand which is the right chain to query
        },
      });
    client.terra = lcd;

    //LocalTerra pre-configured accounts can be checked here https://docs.terra.money/develop/localterra/accounts
    const mk = new MnemonicKey({
        mnemonic: 'notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius',
      });
    client.wallet = lcd.wallet(mk);
    return client
}

export function writeArtifact(data: object, name: string = 'artifact') {
    writeFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), JSON.stringify(data, null, 2))
}

// Tequila lcd is load balanced, so txs can't be sent too fast, otherwise account sequence queries
// may resolve an older state depending on which lcd you end up with. Generally 1000 ms is is enough
// for all nodes to sync up.
let TIMEOUT = 1000

export function setTimeoutDuration(t: number) {
    TIMEOUT = t
}

export function getTimeoutDuration() {
    return TIMEOUT
}

export async function sleep(timeout: number) {
    await new Promise(resolve => setTimeout(resolve, timeout))
}

export class TransactionError extends CustomError {
    public constructor(
        public code: number,
        public codespace: string | undefined,
        public rawLog: string,
    ) {
        super("transaction failed")
    }
}

export async function createTransaction(wallet: Wallet, msg: Msg) {
    return await wallet.createAndSignTx({ msgs: [msg],
        feeDenoms: ['uluna'],
        chainID: 'localterra',
    })
}

export async function broadcastTransaction(terra: LCDClient, signedTx: Tx) {
    const result = await terra.tx.broadcast(signedTx, 'localterra')
    await sleep(TIMEOUT)
    return result
}

export async function performTransaction(terra: LCDClient, wallet: Wallet, msg: Msg) {
    const tx = await createTransaction(wallet, msg)
    const txResult = await broadcastTransaction(terra, tx)

    //Remove the comment to see the raw log of the transaction
    //console.log(txResult);

    if (isTxError(txResult)) {
        throw new Error(
          `encountered an error while running the transaction: ${txResult.code} ${txResult.codespace}`,
        );
      }
    return txResult
}

export async function uploadContract(terra: LCDClient, wallet: Wallet, filepath: string) {
    const storeCode = new MsgStoreCode(
        wallet.key.accAddress('terra'),
        readFileSync(filepath).toString('base64'),
    );
    const storeCodeTx = await wallet.createAndSignTx({
        msgs: [storeCode],
        chainID: 'localterra',
    });
    const storeCodeTxResult = await terra.tx.broadcast(storeCodeTx, 'localterra');
    
    //Remove the comment to see the raw log of the transaction
    //console.log(storeCodeTxResult);
    
    if (isTxError(storeCodeTxResult)) {
        throw new Error(
        `store code failed. code: ${storeCodeTxResult.code}, codespace: ${storeCodeTxResult.codespace}, raw_log: ${storeCodeTxResult.raw_log}`,
        );
    }
    
    console.log("Code ID: %d", storeCodeTxResult.logs[0].eventsByType.store_code.code_id[0])

    const {
        store_code: { code_id },
    } = storeCodeTxResult.logs[0].eventsByType;

    return Number(storeCodeTxResult.logs[0].eventsByType.store_code.code_id[0])
}

export async function instantiateContract(terra: LCDClient, wallet: Wallet, codeId: number, msg: object, init_coins:Coins.Input, label:string) {
    const instantiateMsg = new MsgInstantiateContract(wallet.key.accAddress('terra'), wallet.key.accAddress('terra'), codeId, msg, init_coins, label);
    let result = await performTransaction(terra, wallet, instantiateMsg)
    const attributes = result.logs[0].events[4].attributes[0]
    return attributes.value // contract address
}

export async function executeContract(terra: LCDClient, wallet: Wallet, contractAddress: string, msg: object, coins?: Coins.Input) {
    const executeMsg = new MsgExecuteContract(wallet.key.accAddress('terra'), contractAddress, msg, coins);
    return await performTransaction(terra, wallet, executeMsg);
}

export async function queryContract(terra: LCDClient, contractAddress: string, query: object): Promise<any> {
    return await terra.wasm.contractQuery(contractAddress, query)
}

export async function deployContract(terra: LCDClient, wallet: Wallet, filepath: string, initMsg: object, init_coins: Coins.Input, label: string) {
    const codeId = await uploadContract(terra, wallet, filepath);
    return await instantiateContract(terra, wallet, codeId, initMsg, init_coins, label);
}

export async function migrate(terra: LCDClient, wallet: Wallet, contractAddress: string, newCodeId: number, msg: object) {
    const migrateMsg = new MsgMigrateContract(wallet.key.accAddress('terra'), contractAddress, newCodeId, msg);
    return await performTransaction(terra, wallet, migrateMsg);
}

export function recover(terra: LCDClient, mnemonic: string) {
    const mk = new MnemonicKey({ mnemonic: mnemonic });
    return terra.wallet(mk);
}

export function initialize(terra: LCDClient) {
    const mk = new MnemonicKey();

    console.log(`Account Address: ${mk.accAddress}`);
    console.log(`MnemonicKey: ${mk.mnemonic}`);

    return terra.wallet(mk);
}

export function toEncodedBinary(object: any) {
    return Buffer.from(JSON.stringify(object)).toString('base64');
}

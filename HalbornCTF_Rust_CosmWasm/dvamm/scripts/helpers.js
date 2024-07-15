var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import 'dotenv/config';
import { isTxError, LCDClient, MnemonicKey, MsgExecuteContract, MsgInstantiateContract, MsgMigrateContract, MsgStoreCode } from '@terra-money/feather.js';
import { readFileSync, writeFileSync, } from 'fs';
import path from 'path';
import { CustomError } from 'ts-custom-error';
export const ARTIFACTS_PATH = '../artifacts';
export function readArtifact(name = 'artifact') {
    try {
        const data = readFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), 'utf8');
        return JSON.parse(data);
    }
    catch (e) {
        return {};
    }
}
export function newClient() {
    const client = {};
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
    return client;
}
export function writeArtifact(data, name = 'artifact') {
    writeFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), JSON.stringify(data, null, 2));
}
// Tequila lcd is load balanced, so txs can't be sent too fast, otherwise account sequence queries
// may resolve an older state depending on which lcd you end up with. Generally 1000 ms is is enough
// for all nodes to sync up.
let TIMEOUT = 1000;
export function setTimeoutDuration(t) {
    TIMEOUT = t;
}
export function getTimeoutDuration() {
    return TIMEOUT;
}
export function sleep(timeout) {
    return __awaiter(this, void 0, void 0, function* () {
        yield new Promise(resolve => setTimeout(resolve, timeout));
    });
}
export class TransactionError extends CustomError {
    constructor(code, codespace, rawLog) {
        super("transaction failed");
        this.code = code;
        this.codespace = codespace;
        this.rawLog = rawLog;
    }
}
export function createTransaction(wallet, msg) {
    return __awaiter(this, void 0, void 0, function* () {
        return yield wallet.createAndSignTx({ msgs: [msg],
            feeDenoms: ['uluna'],
            chainID: 'localterra',
        });
    });
}
export function broadcastTransaction(terra, signedTx) {
    return __awaiter(this, void 0, void 0, function* () {
        const result = yield terra.tx.broadcast(signedTx, 'localterra');
        yield sleep(TIMEOUT);
        return result;
    });
}
export function performTransaction(terra, wallet, msg) {
    return __awaiter(this, void 0, void 0, function* () {
        const tx = yield createTransaction(wallet, msg);
        const txResult = yield broadcastTransaction(terra, tx);
        //Remove the comment to see the raw log of the transaction
        //console.log(txResult);
        if (isTxError(txResult)) {
            throw new Error(`encountered an error while running the transaction: ${txResult.code} ${txResult.codespace}`);
        }
        return txResult;
    });
}
export function uploadContract(terra, wallet, filepath) {
    return __awaiter(this, void 0, void 0, function* () {
        const storeCode = new MsgStoreCode(wallet.key.accAddress('terra'), readFileSync(filepath).toString('base64'));
        const storeCodeTx = yield wallet.createAndSignTx({
            msgs: [storeCode],
            chainID: 'localterra',
        });
        const storeCodeTxResult = yield terra.tx.broadcast(storeCodeTx, 'localterra');
        //Remove the comment to see the raw log of the transaction
        //console.log(storeCodeTxResult);
        if (isTxError(storeCodeTxResult)) {
            throw new Error(`store code failed. code: ${storeCodeTxResult.code}, codespace: ${storeCodeTxResult.codespace}, raw_log: ${storeCodeTxResult.raw_log}`);
        }
        console.log("Code ID: %d", storeCodeTxResult.logs[0].eventsByType.store_code.code_id[0]);
        const { store_code: { code_id }, } = storeCodeTxResult.logs[0].eventsByType;
        return Number(storeCodeTxResult.logs[0].eventsByType.store_code.code_id[0]);
    });
}
export function instantiateContract(terra, wallet, codeId, msg, init_coins, label) {
    return __awaiter(this, void 0, void 0, function* () {
        const instantiateMsg = new MsgInstantiateContract(wallet.key.accAddress('terra'), wallet.key.accAddress('terra'), codeId, msg, init_coins, label);
        let result = yield performTransaction(terra, wallet, instantiateMsg);
        const attributes = result.logs[0].events[4].attributes[0];
        return attributes.value; // contract address
    });
}
export function executeContract(terra, wallet, contractAddress, msg, coins) {
    return __awaiter(this, void 0, void 0, function* () {
        const executeMsg = new MsgExecuteContract(wallet.key.accAddress('terra'), contractAddress, msg, coins);
        return yield performTransaction(terra, wallet, executeMsg);
    });
}
export function queryContract(terra, contractAddress, query) {
    return __awaiter(this, void 0, void 0, function* () {
        return yield terra.wasm.contractQuery(contractAddress, query);
    });
}
export function deployContract(terra, wallet, filepath, initMsg, init_coins, label) {
    return __awaiter(this, void 0, void 0, function* () {
        const codeId = yield uploadContract(terra, wallet, filepath);
        return yield instantiateContract(terra, wallet, codeId, initMsg, init_coins, label);
    });
}
export function migrate(terra, wallet, contractAddress, newCodeId, msg) {
    return __awaiter(this, void 0, void 0, function* () {
        const migrateMsg = new MsgMigrateContract(wallet.key.accAddress('terra'), contractAddress, newCodeId, msg);
        return yield performTransaction(terra, wallet, migrateMsg);
    });
}
export function recover(terra, mnemonic) {
    const mk = new MnemonicKey({ mnemonic: mnemonic });
    return terra.wallet(mk);
}
export function initialize(terra) {
    const mk = new MnemonicKey();
    console.log(`Account Address: ${mk.accAddress}`);
    console.log(`MnemonicKey: ${mk.mnemonic}`);
    return terra.wallet(mk);
}
export function toEncodedBinary(object) {
    return Buffer.from(JSON.stringify(object)).toString('base64');
}

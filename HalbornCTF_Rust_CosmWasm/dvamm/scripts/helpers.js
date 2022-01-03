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
import { isTxError, LCDClient, LocalTerra, MnemonicKey, MsgExecuteContract, MsgInstantiateContract, MsgMigrateContract, MsgStoreCode } from '@terra-money/terra.js';
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
    if (process.env.WALLET) {
        client.terra = new LCDClient({
            URL: String(process.env.LCD_CLIENT_URL),
            chainID: String(process.env.CHAIN_ID)
        });
        client.wallet = recover(client.terra, process.env.WALLET);
    }
    else {
        client.terra = new LocalTerra();
        client.wallet = client.terra.wallets.test1;
    }
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
        return yield wallet.createTx({ msgs: [msg] });
    });
}
export function broadcastTransaction(terra, signedTx) {
    return __awaiter(this, void 0, void 0, function* () {
        const result = yield terra.tx.broadcast(signedTx);
        yield sleep(TIMEOUT);
        return result;
    });
}
export function performTransaction(terra, wallet, msg) {
    return __awaiter(this, void 0, void 0, function* () {
        const tx = yield createTransaction(wallet, msg);
        const signedTx = yield wallet.key.signTx(tx);
        const result = yield broadcastTransaction(terra, signedTx);
        if (isTxError(result)) {
            throw new TransactionError(result.code, result.codespace, result.raw_log);
        }
        return result;
    });
}
export function uploadContract(terra, wallet, filepath) {
    return __awaiter(this, void 0, void 0, function* () {
        const contract = readFileSync(filepath, 'base64');
        const uploadMsg = new MsgStoreCode(wallet.key.accAddress, contract);
        let result = yield performTransaction(terra, wallet, uploadMsg);
        return Number(result.logs[0].eventsByType.store_code.code_id[0]); // code_id
    });
}
export function instantiateContract(terra, wallet, codeId, msg) {
    return __awaiter(this, void 0, void 0, function* () {
        const instantiateMsg = new MsgInstantiateContract(wallet.key.accAddress, wallet.key.accAddress, codeId, msg, undefined);
        let result = yield performTransaction(terra, wallet, instantiateMsg);
        const attributes = result.logs[0].events[0].attributes;
        return attributes[attributes.length - 1].value; // contract address
    });
}
export function executeContract(terra, wallet, contractAddress, msg, coins) {
    return __awaiter(this, void 0, void 0, function* () {
        const executeMsg = new MsgExecuteContract(wallet.key.accAddress, contractAddress, msg, coins);
        return yield performTransaction(terra, wallet, executeMsg);
    });
}
export function queryContract(terra, contractAddress, query) {
    return __awaiter(this, void 0, void 0, function* () {
        return yield terra.wasm.contractQuery(contractAddress, query);
    });
}
export function deployContract(terra, wallet, filepath, initMsg) {
    return __awaiter(this, void 0, void 0, function* () {
        const codeId = yield uploadContract(terra, wallet, filepath);
        return yield instantiateContract(terra, wallet, codeId, initMsg);
    });
}
export function migrate(terra, wallet, contractAddress, newCodeId, msg) {
    return __awaiter(this, void 0, void 0, function* () {
        const migrateMsg = new MsgMigrateContract(wallet.key.accAddress, contractAddress, newCodeId, msg);
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

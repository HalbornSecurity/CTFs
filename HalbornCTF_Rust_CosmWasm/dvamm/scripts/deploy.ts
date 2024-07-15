import 'dotenv/config'
import {
    newClient,
    writeArtifact,
    readArtifact,
    deployContract,
    uploadContract,
    instantiateContract,
    executeContract,
    queryContract
} from './helpers.js'
import {configDefault} from './deploy_configs.js'

const TOKEN_INITIAL_AMOUNT = String(1000000000000000)

async function main() {
	
    const {terra, wallet} = newClient();
    console.log(`\nChainID: ${terra.config['localterra'].chainID}\n`)
 
    const user = 'terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n'

    let deployConfig: Config = configDefault
    const network = readArtifact(terra.config['localterra'].chainID)
	
    /*************************************** Register Token Contract *****************************************/
    console.log('Registering CW20 Token Contract...')
    network.tokenCodeID = await uploadContract(terra, wallet, '../artifacts/cw20_base.wasm')

    /*************************************** Register Pairs Contract *****************************************/
    console.log('Registering Pair Contract...')
    network.pairCodeID = await uploadContract(terra, wallet, '../artifacts/dvamm_pair.wasm')

    /*************************************** Deploy Factory Contract *****************************************/
    console.log('Deploying Factory...')
    deployConfig.factoryInitMsg.config.pair_configs[0].code_id = network.pairCodeID
    deployConfig.factoryInitMsg.config.token_code_id = network.tokenCodeID
    deployConfig.factoryInitMsg.config.owner = wallet.key.accAddress('terra')
    network.factoryAddress = await deployContract(
        terra,
        wallet,
        '../artifacts/dvamm_factory.wasm',
        deployConfig.factoryInitMsg.config,
        "1000000000uluna",
        "factory"
    )
    console.log(`Factory Contract Address: ${network.factoryAddress}`)
   
    let handleMsg = {   update_config: {
                            pair_xyk_config: {
                                code_id: 2,
                                total_fee_bps: 1,
                                maker_fee_bps: 1,
                            }
                    } }; 

    await executeContract(terra, wallet, network.factoryAddress, handleMsg );

    writeArtifact(network, terra.config['localterra'].chainID)

    /*************************************** Deploy Dummy Tokens *****************************************/
    console.log('\nDeploying Dummy Tokens...')

    // Token A info
    let TOKEN_NAME = "Token A";
    let TOKEN_SYMBOL = "TOK-A";
    let TOKEN_DECIMALS = 6;
    let TOKEN_INFO = {
        name: TOKEN_NAME,
        symbol: TOKEN_SYMBOL,
        decimals: TOKEN_DECIMALS,
        initial_balances: [
            {
                address: user,
                amount: TOKEN_INITIAL_AMOUNT
            }
        ]
    };

    // Instantiate token A contract
    let tokenA = await instantiateContract(terra, wallet, network.tokenCodeID, TOKEN_INFO, "1000000uluna","tokenA");
    console.log('Token A Address: ' + tokenA);


    // Token B info
    TOKEN_NAME = "Token B";
    TOKEN_SYMBOL = "TOK-B";
 
    TOKEN_INFO = {
        name: TOKEN_NAME,
        symbol: TOKEN_SYMBOL,
        decimals: TOKEN_DECIMALS,
        initial_balances: [
            {
                address: user,
                amount: TOKEN_INITIAL_AMOUNT
            }
        ]
    };

    let tokenB = await instantiateContract(terra, wallet, network.tokenCodeID, TOKEN_INFO, "1000000uluna","tokenB");
    console.log('Token B Address: ' + tokenB);

    console.log(`\nADMIN wallet: ${wallet.key.accAddress('terra')}`)
    console.log(`USER wallet: ${user}\n`)

    console.log('Balance for USER:');
    let balanceA = await queryContract(terra, tokenA, { balance: { address: user} })
    let balanceB = await queryContract(terra, tokenB, { balance: { address: user} })

    console.log(' + ' + balanceA.balance + ' tokens A');
    console.log(' + ' + balanceB.balance + ' tokens B');
    console.log("");
    console.log('\Setup is complete... now it is HACKING time!!')
}

main().catch(console.log)

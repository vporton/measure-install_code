import { HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { createActor as createMeasureActor,  } from "./src/declarations/measure_backend";
import { Measure } from "./src/declarations/measure_backend/measure_backend.did";
import { configDotenv } from "dotenv";
import { appendFile, readFile } from "fs/promises";
import {Ed25519KeyIdentity} from '@dfinity/identity';
import {Secp256k1KeyIdentity} from '@dfinity/identity-secp256k1';
import { decode } from 'pem-file';
import { exec } from "child_process";
import { ICManagementCanister } from "@dfinity/ic-management";

configDotenv({});

export function commandOutput(command: string): Promise<string> {
    return new Promise((resolve) => exec(command, function(error, stdout, stderr){ resolve(stdout); }));
}

export function decodeFile(rawKey) {
    let buf/*: Buffer*/ = decode(rawKey);
    if (rawKey.includes('EC PRIVATE KEY')) {
        if (buf.length != 118) {
            throw 'expecting byte length 118 but got ' + buf.length;
        }
        return Secp256k1KeyIdentity.fromSecretKey(buf.subarray(7, 39));
    }
    if (buf.length != 85) {
        throw 'expecting byte length 85 but got ' + buf.length;
    }
    let secretKey = Buffer.concat([buf.subarray(16, 48), buf.subarray(53, 85)]);
    const identity = Ed25519KeyIdentity.fromSecretKey(secretKey);
    return identity;
}

async function main() {
    for (let i = 2; i < process.argv.length; ++i) {
        const wasmFileName = process.argv[i];
        const key = await commandOutput("dfx identity export `dfx identity whoami`"); // secret key
        const identity = decodeFile(key);
        const user = identity.getPrincipal();

        const agent = await HttpAgent.create({host: "http://localhost:8080", identity, shouldFetchRootKey: true});

        const { updateSettings } = ICManagementCanister.create({
            agent,
        });

        await updateSettings({
            canisterId: Principal.fromText(process.env.CANISTER_ID_DUMMY!),
            settings: {
                controllers: [user.toText(), process.env.CANISTER_ID_MEASURE_BACKEND!],
            },
        });

        const actor: Measure = createMeasureActor(process.env.CANISTER_ID_MEASURE_BACKEND!, {agent});
        const wasm = await readFile(wasmFileName);
        const res = await actor.main(wasm);
        const res2 = {...res, file: wasmFileName};
        console.log(res2);
        await appendFile('measures.log', `${res.moduleSize} ${res.cyclesSpent}\n`);
    }
}

(async () => main())()
import { HttpAgent } from "@dfinity/agent";
import { createActor as createMeasureActor,  } from "./src/declarations/measure_backend";
import { Measure } from "./src/declarations/measure_backend/measure_backend.did";
import { configDotenv } from "dotenv";
import { readFile } from "fs/promises";

configDotenv({});

async function main() {
    const wasmFileName = process.argv[1];
    const agent = await HttpAgent.create({host: "http://localhost:8080"});
    const actor: Measure = createMeasureActor(process.env.CANISTER_ID_MEASURE_BACKEND!, {agent});
    const wasm = await readFile(wasmFileName);
    const res = await actor.main(wasm);
    console.log(res);
}

(async () => main())()
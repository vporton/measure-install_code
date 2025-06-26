import EC "mo:base/ExperimentalCycles";
import Principal "mo:base/Principal";
import {ic} "mo:ic";
import Dummy "canister:dummy";

actor class Measure() {
  type ReturnType = {
    cyclesSpent: Nat;
    moduleSize: Nat;
  };

  public shared func main(wasm: Blob) : async ReturnType {
    let balance = EC.balance();
    await ic.install_code({
      arg = to_candid({});
      wasm_module = wasm;
      mode = #reinstall;
      canister_id = Principal.fromActor(Dummy);
      sender_canister_version = null;
    });
    {
      cyclesSpent = balance - EC.balance();
      moduleSize = wasm.size();
    };
  };
};

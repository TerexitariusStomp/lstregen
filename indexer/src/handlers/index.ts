/**
 * Handlers for processing decoded contract events or messages.
 * In more advanced setups, you would parse wasm event attributes into typed
 * domain events here and forward them to persistence or downstream buses.
 */

export type WasmEventAttr = { key: string; value: string };
export type WasmEvent = { type: string; attributes: WasmEventAttr[] };

export function extractAttribute(event: WasmEvent, key: string): string | undefined {
  return event.attributes.find((a) => a.key === key)?.value;
}

export function isContractEventForAddress(event: WasmEvent, contractAddress: string): boolean {
  return !!event.attributes.find(
    (a) => a.key === '_contract_address' && a.value === contractAddress,
  );
}
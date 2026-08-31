# alux-jsonrpc-typescript

`alux-jsonrpc-typescript` interprets an [`alux-jsonrpc`](https://docs.rs/alux-jsonrpc) program as a
TypeScript client module.

The program states a method's name, its argument product and its answer. The shapes of that product
and that answer, read through [`alux-shape`](../../alux-shape), state their types. Folding the two
together is the whole of a client, so no part of the surface is written a second time in another
language.

```rust ignore
use alux_jsonrpc::JsonRpcProgramExt;
use alux_jsonrpc_typescript::TsClient;
use alux_shape::Spelling;

let ts = TsClient::new(Spelling::LowerCamel);

println!("{}", ts.compile_jsonrpc(ts.user_rpc::<Directory>()).render());
```

```ts
export interface User {
  id: number
  displayName: string
  email: string | null
}

export const program = {
  user_byId: method<[userId: number], User | null>("user_byId", []),
  user_count: method<[], number>("user_count", []),
  user_getById: method<[userId: number], User | null>("user_getById", ["user_id"]),
} as const
```

Each entry is a first-order descriptor carrying its argument product and its answer, so an
interpretation of the module — a promise client, a hook adapter, a mock, a printed description — is
written once for every method there is or will be, rather than once per method.

A fallible method states only the value it answers with. Its failure reaches a caller as a rejected
call, which is what `OutcomeAlg` already says about a domain error.

## What the parameter mode changes, and what it does not

Every parameter is labelled, whichever mode the method uses, because `OperationAlg::ARG_NAMES` holds
the authored names for every operation and both registrations state them. So `user_byId` above reads
`method<[userId: number], …>` even though it is decoded from an array.

What the mode changes is the request document, which is the second argument of each entry. A method
decoded from a parameter object lists its names there, `["user_id"]`, and the client sends an object. A
method decoded from an array lists none, `[]`, and the client sends the values in order. The label is
for whoever writes the call; the list is what goes on the wire.

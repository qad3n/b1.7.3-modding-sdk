# Contributing

This is a personal project. Pull requests and outside issues are not accepted, and pull requests will be closed
without review.

You are welcome to fork it under the terms of the license and change your fork however you like.

If you use the `/modport` skill or an AI agent with this code, keep its changes in your own working tree or fork.
Do not point it at this repository to open pull requests or issues.

## Where code goes

| Code | Place |
|---|---|
| Decoding host replies | `b173_api/src/reply.rs` (`expect*`, `fetchData`), fire and forget calls through `rt::fire` |
| Builder steps that set one spec field | the `setters!` block of that builder (`b173_api/src/macros.rs`) |
| Registration, one area per file | `b173_api/src/register/` (blocks, items, entities, recipes, net, ui, services, content, world) |
| Shared vector, block position and angle maths | `b173_api/src/math.rs`, faces as `math::Face` |
| Faithful vanilla generator ports | `b173_api/src/terrain/` |
| ABI types | `mod_protocol/src/abi.rs`, new fields only as new variants or structs |

# NEAR Non Fungible Token With Clone

## Summary
Extended version of non-fungible tokens (NEP-171) that allows cloning tokens from existing ones.
## Motivation
In Near blockchain there is a concept of storage cost, each byte added to a contract requires [storage stake](https://docs.near.org/concepts/storage/storage-staking), the NEP-171 allows to mint NFTs but sometimes those NFTs are only duplicates with same metadata that still require to cover the storage stake.
## Rationale and alternatives
NEAR's core Non-Fungible Token standard supports minting NFTs, when the metadata extension is enabled the an NFT can stores metadata. The NEP-171 allows to mint NFTs and set the metadata but there is no way to share/reuse the metadata between NFTs to reduce the storage stake.
Prior art:
- Non-Fungible Token standard (NEP-171)

## Specification
### Example Scenarios
https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/metadata.rs#L27
Let's say we have a collecion of NFTs where some of those NFTs are only duplicates(copies) and we want to mint `1000 NFT` that use the same metadata. The total size of the metadata is `1000 byte`. 
The cost to store 1 Byte in a near account is `10^19 yocto NEAR/byte`

1. Minting:
To mint those `1000 NFTs` we have to cover a total storage cost:
    ```js
    totalCost = 1000 copies * (1000 byte * 10^19 yocto NEAR/byte)
    totalCost = 10 NEAR
    ```
 2. Cloning 
 Now, let's compare if we use the clone feature
     ```js
    totalCost = (1000 byte * 10^19 yocto NEAR/byte) + (1000 clones * (0))
    totalCost = 0.01 NEAR
    ```
### [internal_clone_mint](https://github.com/Shard-Labs/near_non_fungible_token_with_clone/blob/main/src/non_fungible_token_clone/core/core_impl.rs#L48)
Input 
```rs
pub fn internal_clone_mint(
    &mut self,
    token_id: TokenId,
    clone_from_id: TokenId,
    token_owner_id: AccountId,
) -> Token;
```
The function accept as input a `clone_from_id`, as a result a new NFT is cloned and linked with a parent NFT using [nft_clone_from_id](https://github.com/Shard-Labs/near_non_fungible_token_with_clone/blob/main/src/non_fungible_token_clone/core/core_impl.rs#L13)


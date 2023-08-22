use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::events::NftMint;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap};
use near_sdk::{env, AccountId, IntoStorageKey, PromiseOrValue};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleTokenClone {
    pub nft: NonFungibleToken,
    pub nft_clone_from_id: TreeMap<TokenId, TokenId>,
    pub nft_clone_count: LookupMap<TokenId, u128>,
}

impl NonFungibleTokenClone {
    pub fn new<Q, R, S, T, P, C>(
        owner_by_id_prefix: Q,
        owner_id: AccountId,
        token_metadata_prefix: Option<R>,
        enumeration_prefix: Option<S>,
        approval_prefix: Option<T>,
        clone_prefix: P,
        clone_count_prefix: C,
    ) -> Self
    where
        Q: IntoStorageKey,
        R: IntoStorageKey,
        S: IntoStorageKey,
        T: IntoStorageKey,
        P: IntoStorageKey,
        C: IntoStorageKey,
    {
        NonFungibleTokenClone {
            nft: NonFungibleToken::new(
                owner_by_id_prefix,
                owner_id,
                token_metadata_prefix,
                enumeration_prefix,
                approval_prefix,
            ),
            nft_clone_from_id: TreeMap::new(clone_prefix),
            nft_clone_count: LookupMap::new(clone_count_prefix),
        }
    }

    pub fn internal_clone_mint(
        &mut self,
        token_id: TokenId,
        clone_from_id: TokenId,
        token_owner_id: AccountId,
    ) -> Token {
        if self.nft.token_metadata_by_id.is_none() {
            panic!("Token metadata extension must be used to clone.")
        }

        let token = self.nft.internal_mint_with_refund(
            token_id.clone(),
            token_owner_id,
            // If not add the metadata the internal_mint_with_refund will panic with "Must provide metadata".
            // ref: core_impls L341
            Some(TokenMetadata {
                title: None,
                description: None,
                media: None,
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            }),
            Some(env::predecessor_account_id()),
        );

        let mut count = self.nft_clone_count.get(&clone_from_id).unwrap_or(0);
        count += 1;
        self.nft_clone_count.insert(&clone_from_id, &count);

        // Delete the metadata added above.
        self.nft
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.remove(&token_id));

        self.nft_clone_from_id.insert(&token_id, &clone_from_id);

        NftMint {
            owner_id: &token.owner_id,
            token_ids: &[&token.token_id],
            memo: None,
        }
        .emit();
        token
    }
}

impl NonFungibleTokenCore for NonFungibleTokenClone {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        self.nft
            .nft_transfer(receiver_id, token_id, approval_id, memo);
    }

    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.nft
            .nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let unwrap_token = self
            .nft
            .nft_token(token_id.clone())
            .unwrap_or_else(|| env::panic_str("Token does not exist"));
        let clone_from = self
            .nft_clone_from_id
            .get(&token_id)
            .unwrap_or_else(|| token_id);
        let metadata = self
            .nft
            .token_metadata_by_id
            .as_ref()
            .unwrap_or_else(|| env::panic_str("Token not found within metadata"))
            .get(&clone_from);
        Some(Token {
            token_id: unwrap_token.token_id,
            owner_id: unwrap_token.owner_id,
            metadata: metadata,
            approved_account_ids: unwrap_token.approved_account_ids,
        })
    }
}

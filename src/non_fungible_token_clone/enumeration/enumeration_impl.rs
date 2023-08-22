use crate::non_fungible_token_clone::NonFungibleTokenClone;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::json_types::U128;
use near_sdk::{env, require, AccountId};

type TokenId = String;

impl NonFungibleTokenClone {
    /// Helper function used by a enumerations methods
    /// Note: this method is not exposed publicly to end users
    fn enum_get_token(&self, owner_id: AccountId, token_id: TokenId) -> Token {
        // If token_id is not found within nft_clone_from_id it means token_id is a genesis token
        let clone_id = self
            .nft_clone_from_id
            .get(&token_id)
            .unwrap_or_else(|| token_id.clone());

        let metadata = self
            .nft
            .token_metadata_by_id
            .as_ref()
            .and_then(|m| m.get(&clone_id));

        let approved_account_ids = self.nft.approvals_by_id.as_ref().map(|approvals_by_id| {
            approvals_by_id
                .get(&token_id.to_string())
                .unwrap_or_default()
        });

        Token {
            token_id,
            owner_id,
            metadata,
            approved_account_ids,
        }
    }
}

impl NonFungibleTokenEnumeration for NonFungibleTokenClone {
    fn nft_total_supply(&self) -> U128 {
        self.nft.nft_total_supply()
    }

    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Token> {
        // Get starting index, whether or not it was explicitly given.
        // Defaults to 0 based on the spec:
        // https://nomicon.io/Standards/NonFungibleToken/Enumeration.html#interface
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        require!(
            (self.nft.owner_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        require!(limit != 0, "Cannot provide limit of 0.");
        self.nft
            .owner_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_id, owner_id)| self.enum_get_token(owner_id, token_id))
            .collect()
    }

    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        self.nft.nft_supply_for_owner(account_id)
    }

    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token> {
        let tokens_per_owner = self.nft.tokens_per_owner.as_ref().unwrap_or_else(|| {
            env::panic_str(
                "Could not find tokens_per_owner when calling a method on the \
                enumeration standard.",
            )
        });
        let token_set = if let Some(token_set) = tokens_per_owner.get(&account_id) {
            token_set
        } else {
            return vec![];
        };
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        require!(limit != 0, "Cannot provide limit of 0.");
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        require!(
            token_set.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        token_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|token_id| self.enum_get_token(account_id.clone(), token_id))
            .collect()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::non_fungible_token_clone::NonFungibleTokenClone;
    use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
    use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
    use near_contract_standards::non_fungible_token::{Token, TokenId};
    use near_sdk::borsh::{self, BorshSerialize};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env, BorshStorageKey};

    #[test]
    fn test_clone_many() {
        let mut ctx = VMContextBuilder::new();
        ctx.context.attached_deposit = 7000000000000000000000;
        testing_env!(ctx.context);

        #[derive(BorshStorageKey, BorshSerialize)]
        pub enum StorageKey {
            NonFungibleToken,
            TokenMetadata,
            Enumeration,
            Approval,
            OriginClone,
            Clone,
        }

        let mut contract = NonFungibleTokenClone::new(
            StorageKey::NonFungibleToken,
            env::current_account_id(),
            Some(StorageKey::TokenMetadata),
            Some(StorageKey::Enumeration),
            Some(StorageKey::Approval),
            StorageKey::OriginClone,
            StorageKey::Clone,
        );

        let clone_from_id: TokenId = "1".into();
        let parent_metadata: TokenMetadata = TokenMetadata {
            title: Some("title".into()),
            description: Some("description".into()),
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
        };

        contract.nft.internal_mint(
            clone_from_id.clone(),
            accounts(0),
            Some(parent_metadata.clone()),
        );

        for index in 0..10 {
            let clone_token_id: TokenId = format!("{}", 2 + index);

            let clone_owner_id = accounts(1);
            contract.internal_clone_mint(
                clone_token_id.clone(),
                clone_from_id.clone(),
                clone_owner_id.clone(),
            );
            assert!(
                contract.nft_clone_count.get(&clone_from_id.clone()) == (index + 1).into(),
                "Invalid clone count GET:{:?}",
                contract.nft_clone_count.get(&clone_from_id.clone())
            );

            let clone: Token = contract
                .nft_token(clone_token_id.clone())
                .unwrap_or_else(|| panic!("Clone token not exist"));

            assert!(
                clone.token_id == clone_token_id,
                "Clone token id doesn't match"
            );

            assert!(
                clone.owner_id == clone_owner_id,
                "Clone token owner doesn't match"
            );

            let clone_metadata = clone.metadata.unwrap();
            assert!(
                clone_metadata.title == parent_metadata.clone().title
                    && clone_metadata.description == parent_metadata.clone().description
                    && clone_metadata.media == parent_metadata.media
                    && clone_metadata.media_hash == parent_metadata.media_hash
                    && clone_metadata.copies == parent_metadata.copies
                    && clone_metadata.issued_at == parent_metadata.issued_at
                    && clone_metadata.expires_at == parent_metadata.expires_at
                    && clone_metadata.starts_at == parent_metadata.starts_at
                    && clone_metadata.updated_at == parent_metadata.updated_at
                    && clone_metadata.extra == parent_metadata.extra
                    && clone_metadata.reference == parent_metadata.reference
                    && clone_metadata.reference_hash == parent_metadata.reference_hash,
                "Clone token metadada doesn't match"
            );
        }

        let tokens = contract.nft_tokens_for_owner(accounts(0), None, None);
        assert_eq!(accounts(0), tokens[0].owner_id, "Owner must be Alice");

        let token = contract.nft_token("1".to_string());
        assert_eq!(accounts(0), token.unwrap().owner_id, "Owner must be Alice");

        let token = contract.nft_token("2".to_string());
        assert_eq!(accounts(1), token.unwrap().owner_id, "Owner must be Bob");

        let contract_enum: Box<dyn NonFungibleTokenEnumeration> =
            Box::new(contract) as Box<dyn NonFungibleTokenEnumeration>;
        assert!(
            contract_enum.nft_total_supply() == 11.into(),
            "Total should be 11 tokens 1 parent and 10 clones"
        );

        let alice_tokens_count = contract_enum.nft_supply_for_owner(accounts(0));
        assert!(
            alice_tokens_count == U128(1 as u128),
            "Total must be 1 but got: {:?}",
            alice_tokens_count
        );

        let bob_tokens_count = contract_enum.nft_supply_for_owner(accounts(1));
        assert!(
            bob_tokens_count == U128(10 as u128),
            "Total must be 10 but got: {:?}",
            bob_tokens_count
        );
        contract_enum.nft_supply_for_owner(accounts(1));
    }

    #[should_panic(expected = "Token does not exist")]
    #[test]
    fn test_token_not_found() {
        let mut ctx = VMContextBuilder::new();
        ctx.context.attached_deposit = 7000000000000000000000;
        testing_env!(ctx.context);

        #[derive(BorshStorageKey, BorshSerialize)]
        pub enum StorageKey {
            NonFungibleToken,
            TokenMetadata,
            Enumeration,
            Approval,
            OriginClone,
            Clone,
        }

        let contract = NonFungibleTokenClone::new(
            StorageKey::NonFungibleToken,
            env::current_account_id(),
            Some(StorageKey::TokenMetadata),
            Some(StorageKey::Enumeration),
            Some(StorageKey::Approval),
            StorageKey::OriginClone,
            StorageKey::Clone,
        );
        contract.nft_token("999".to_string());
    }
}

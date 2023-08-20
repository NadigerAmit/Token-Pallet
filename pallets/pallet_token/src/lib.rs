#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::traits::{Currency};
    use frame_support::{pallet_prelude::Weight, traits::Randomness};
    use frame_system::pallet_prelude::BlockNumberFor;
    use frame_system::Config as SystemConfig;
    use sp_runtime::generic::Block;
    use scale_info::prelude::vec::Vec;

    use sp_io::hashing::blake2_256;
    use frame_system::Module as System;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)] //  This attribute macro is used to generate the storage traits and implementations needed by the pallet.
    pub struct Pallet<T>(_);


    #[pallet::config]
    pub trait Config: frame_system::Config {
		    type Currency: Currency<Self::AccountId>;
        /// Something that provides randomness in the runtime.
		  //  type TokenRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
        

		    #[pallet::constant]
		    type MaximumOwned: Get<u32>;
        
        
        // runtime events : emit events to notify front-end applications about the result of a transaction that
        //  executed successfully. 
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    }

    type BalanceOf<T> =
         <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Clone, Encode, Decode, PartialEq, /*Copy,*/ RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Token<T: Config> {
        // Unsigned integers of 16 bytes to represent a unique identifier
        pub unique_id: [u8; 16],
        // `None` assumes not for sale
        pub price: Option<BalanceOf<T>>,
      // pub color: Color,
        pub owner: T::AccountId,
    }

  	/// The lookup table for tokens.
    #[pallet::storage]
    pub(super) type TokenCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Maps the token struct to the unique_id.
    #[pallet::storage]
    pub(super) type TokenMap<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Token<T>>;
    
    //Mapping owners to their Tokens
    /// Track the Tokens owned by each account.
    #[pallet::storage]
    pub(super) type OwnerOfTokens<T: Config> = StorageMap< // This is defining the storage map provided by FRAME
      _,
      Twox64Concat, // This is the hasher used for key hashing.
      T::AccountId, // Key in the map 
      BoundedVec<[u8; 16], T::MaximumOwned>,  // value in the map 
      ValueQuery, // query type of storage system 
    >;

    #[pallet::error]
    pub enum Error<T> {
      /// Each token must have a unique identifier
      DuplicateToken,

      /// An account can't exceed the `MaximumOwned` constant
      MaximumTokensOwned,

      /// The total supply of tokens can't exceed the u64 limit
      BoundsOverflow,	 
      
      /// The token doesn't exist
      NoToken,

      /// You are not the owner
      NotOwner,

      /// Trying to transfer a token to yourself
      TransferToSelf,

      	/// The bid is lower than the asking price.
      BidPriceTooLow,

      /// The token is not for sale.
      NotForSale,

    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
         /// A new token was successfully created
         TokenCreated { 
             Token: [u8; 16], owner: T::AccountId 
         },

         TokenBurned { 
          Token: [u8; 16], owner: T::AccountId 
      },

         TransferSucceeded { 
              from: T::AccountId, 
              to: T::AccountId, 
              token: [u8; 16] 
         },

         PriceSet { 
              token: [u8; 16], 
              price: Option<BalanceOf<T>> 
         },

         /// A tokenm was successfully sold.
         Sold { 
              seller: T::AccountId, 
              buyer: T::AccountId, 
              token: [u8; 16], 
              price: BalanceOf<T> 
         },
    }
    
    // Pallet internal functions
    impl<T: Config> Pallet<T> {
          
          pub fn generate_random_number() -> u64 {
            let current_block = frame_system::Pallet::<T>::block_number();
            let mut input_bytes = Vec::new();
                // Convert the block number to bytes manually
            let block_number_bytes = current_block.encode();
            input_bytes.extend_from_slice(&block_number_bytes);
    
            input_bytes.extend_from_slice(b"random_seed");
            
            let random_seed = blake2_256(&input_bytes);
            let random_number = u64::from_le_bytes(random_seed[0..8].try_into().unwrap());
            random_number
        }


      // Generates and returns the unique_id of the token 
          fn gen_unique_id() -> [u8; 16] {
              let random = Self::generate_random_number();
             // let random = 10001;
              let unique_payload = (
                  random,
                  frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),frame_system::Pallet::<T>::block_number(),
              );
              // Turns into a byte array
	            let encoded_payload = unique_payload.encode();
              let hash = frame_support::Hashable::blake2_128(&encoded_payload);
              hash
          }

          
          pub fn mint(
            owner: &T::AccountId,
            unique_id: [u8; 16],
          ) -> Result<[u8; 16], DispatchError> {
              let token  = Token::<T> { unique_id, price: None, owner: owner.clone() };
      
              // Check if the token  exists in the storage map
              ensure!(!TokenMap::<T>::contains_key(&token.unique_id), Error::<T>::DuplicateToken);
      
             // Check that a new token  can be created
             let count = TokenCount::<T>::get();
             let new_count = count.checked_add(1).ok_or(Error::<T>::BoundsOverflow)?;
            
             // Append token to OwnerOfTokens map
             OwnerOfTokens::<T>::try_append(&owner, token.unique_id)
			            .map_err(|_| Error::<T>::MaximumTokensOwned)?;

            // Write new token to storage and update the count
            TokenMap::<T>::insert(token.unique_id, token);
            TokenCount::<T>::put(new_count);

            // Deposit the "ToeknCreated" event.
            //deposit_event() is a macro defined in the frame_system pallet
		        Self::deposit_event(Event::TokenCreated {Token:unique_id, owner: owner.clone() });
      
             Ok(unique_id)
          }

          pub fn burn(
            requester: T::AccountId,
            token_id: [u8; 16],
          ) -> Result<[u8; 16], DispatchError> {
              // Get the token 
              let mut token = TokenMap::<T>::get(&token_id).ok_or(Error::<T>::NoToken)?;
              let owner = token.owner;
              ensure!(requester == owner, Error::<T>::NotOwner);

              let mut from_owned = OwnerOfTokens::<T>::get(&owner);
              // Remove token from owned tokens.
              if let Some(ind) = from_owned.iter().position(|&id| id == token_id) {
                // swap_remove() swapping the element with the last element and then removing the last element. 
                from_owned.remove(ind);
              } else {
                return Err(Error::<T>::NoToken.into())
              }
              
              //from_owned.remove(token_id);
              let count = TokenCount::<T>::get();

              OwnerOfTokens::<T>::insert(&owner, from_owned);
              TokenCount::<T>::put(count-1);
              <TokenMap<T>>::take(token_id);
              Self::deposit_event(Event::TokenBurned { Token: token_id,owner: owner.clone() });
              Ok(token_id) 
          }

          // Update storage to transfer token
          pub fn do_transfer(
            token_id: [u8; 16],
            to: T::AccountId,
          ) -> DispatchResult {
            	 // Get the token 
              let mut token = TokenMap::<T>::get(&token_id).ok_or(Error::<T>::NoToken)?;
              let from = token.owner;

              ensure!(from != to, Error::<T>::TransferToSelf);
              let mut from_owned = OwnerOfTokens::<T>::get(&from);

              // Remove token  from list of owned tokens.
              if let Some(ind) = from_owned.iter().position(|&id| id == token_id) {
                from_owned.swap_remove(ind);
              } else {
                return Err(Error::<T>::NoToken.into())
              }

              // Add token to the list of owned tokens.
              let mut to_owned = OwnerOfTokens::<T>::get(&to);
              to_owned.try_push(token_id).map_err(|_id| Error::<T>::MaximumTokensOwned)?;

              // Transfer succeeded, update the owner and reset the price to `None`.
              token.owner = to.clone();
              token.price = None;

              // Write updates to storage
              TokenMap::<T>::insert(&token_id, token);
              OwnerOfTokens::<T>::insert(&to, to_owned);
              OwnerOfTokens::<T>::insert(&from, from_owned);
              
              Self::deposit_event(Event::TransferSucceeded { from, to, token: token_id });
              Ok(())
          }

          // An internal function for purchasing a token
          pub fn do_buy_token(
            unique_id: [u8; 16],
            to: T::AccountId,
            bid_price: BalanceOf<T>,
          ) -> DispatchResult {

              // Get the token from the storage map
              let mut token = TokenMap::<T>::get(&unique_id).ok_or(Error::<T>::NoToken)?;
              let from = token.owner;
              ensure!(from != to, Error::<T>::TransferToSelf);
              let mut from_owned = OwnerOfTokens::<T>::get(&from);

              // Remove token from owned tokens.
              if let Some(ind) = from_owned.iter().position(|&id| id == unique_id) {
                // swap_remove() swapping the element with the last element and then removing the last element. 
                from_owned.swap_remove(ind);
              } else {
                return Err(Error::<T>::NoToken.into())
              }

              // Add Token to owned tokens.
              let mut to_owned = OwnerOfTokens::<T>::get(&to);
              to_owned.try_push(unique_id).map_err(|_id| Error::<T>::MaximumTokensOwned)?;

              // Mutating state with a balance transfer, so nothing is allowed to fail after this.
              if let Some(price) = token.price {
                  ensure!(bid_price >= price, Error::<T>::BidPriceTooLow);
                  // Transfer the amount from buyer to seller
                  T::Currency::transfer(&to, &from, price, frame_support::traits::ExistenceRequirement::KeepAlive)?;
                  
                  // Deposit sold event
                  Self::deposit_event(Event::Sold {
                    seller: from.clone(),
                    buyer: to.clone(),
                    token: unique_id,
                    price,
                  });
              } else {
                  return Err(Error::<T>::NotForSale.into())
              }
              
              // Transfer succeeded, update the token owner and reset the price to `None`.
              token.owner = to.clone();
              token.price = None;

              // Write updates to storage
              TokenMap::<T>::insert(&unique_id, token);
              OwnerOfTokens::<T>::insert(&to, to_owned);
              OwnerOfTokens::<T>::insert(&from, from_owned);
              Self::deposit_event(Event::TransferSucceeded { from, to, token: unique_id });
              Ok(())
          }    
    }

    // Create the callable function that uses the internal functions.

    // Pallet callable functions
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new unique Token.
        /// The actual token creation is done in the `mint()` function.
        #[pallet::weight(0)]
        pub fn create_token(origin: OriginFor<T>) -> DispatchResult {
            // Make sure the caller is from a signed origin
            let sender = ensure_signed(origin)?;
            // Generate the unique_id using a helper function
            let token_gen_unique_id = Self::gen_unique_id();
            // Write new token  to storage by calling helper function
            Self::mint(&sender, token_gen_unique_id)?;
            Ok(())
        }

        /// Brun the already created Token.
        /// The actual token deletion is done in the `burn()` function.
        #[pallet::weight(0)]
        pub fn burn_token(
          origin: OriginFor<T>,
          unique_id: [u8; 16],) -> DispatchResult {
            // Make sure the caller is from a signed origin
            let sender = ensure_signed(origin)?;
            Self::burn(sender,unique_id)?;
            Ok(())
        }

        /// Transfer a token to another account.
        /// Any account that holds a token can send it to another account. 
        /// Transfer resets the price of the token, marking it not for sale.

        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            unique_id: [u8; 16],
        ) -> DispatchResult {
            // Make sure the caller is from a signed origin
            let from = ensure_signed(origin)?;
            let token = TokenMap::<T>::get(&unique_id).ok_or(Error::<T>::NoToken)?;
            ensure!(token.owner == from, Error::<T>::NotOwner);
            Self::do_transfer(unique_id, to)?;
            Ok(())
        }

        // Update the tToken price and write to storage.
        /*
           Need to implement below functionality:
            The caller must be a signed origin.
            The Token  must already exist.
            The caller must be the owner of the Token.
        */
        #[pallet::weight(0)]
        pub fn set_price(
            origin: OriginFor<T>,
            unique_id: [u8; 16],
            new_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            // Make sure the caller is from a signed origin
            let sender = ensure_signed(origin)?;

            // Ensure the token exists and is called by the owner
            let mut token = TokenMap::<T>::get(&unique_id).ok_or(Error::<T>::NoToken)?;
            ensure!(token.owner == sender, Error::<T>::NotOwner);

            // Set the price in storage
            token.price = new_price;
            TokenMap::<T>::insert(&unique_id, token);

            // Deposit a "PriceSet" event.
            Self::deposit_event(Event::PriceSet { token: unique_id, price: new_price });

            Ok(())
        }

        /*
          The proposed buying price is greater than or equal to the price set for the token by its owner and returns the BidPriceTooLow error if the proposed price is too low.
          The token is for sale and returns a NotForSale error if the token price is None.
          The account for the buyer has a free balance available to cover the price set for the token.
          The account for the buyer doesn't already own too many token to receive another token.
        */
        /// Buy a token. The bid price must be greater than or equal to the price
        /// set by the token owner.
        #[pallet::weight(0)]
        pub fn buy_token(
          origin: OriginFor<T>,
          unique_id: [u8; 16],
          bid_price: BalanceOf<T>,
        ) -> DispatchResult {
              // Make sure the caller is from a signed origin
              let buyer = ensure_signed(origin)?;
              // Transfer the token from seller to buyer.
              Self::do_buy_token(unique_id, buyer, bid_price)?;
              Ok(())
        }
    }

}
#![cfg_attr(not(feature ="std"),no_std)]

/// poe 交易存证 by ming

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::IsType, Blake2_128Concat};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

   #[pallet::config] 
   pub trait Config: frame_system::Config{
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }
   #[pallet::event]
   #[pallet::generate_deposit(pub(super) fn deposit_event)]
   pub enum Event<T:Config> {
        ClaimCreated(T::AccountId,Vec<u8>),
        ClaimRevoked(T::AccountId,Vec<u8>),
        ClaimMoved(T::AccountId,Vec<u8>),
    }
   #[pallet::error]
   pub enum Error<T> {
        ProofAlreadyClaimed,
        NoSuchProof,
        NotProofOwner,
        SameOwner,
   }

   #[pallet::pallet]
   #[pallet::generate_store(pub(super) trait Store)]
   pub struct Pallet<T>(_);

   #[pallet::storage]
   pub(super) type Proofs<T:Config> = StorageMap<
    _,
    Blake2_128Concat,
    Vec<u8>,(T::AccountId,T::BlockNumber),
    ValueQuery
   >;
   #[pallet::hooks]
   impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
   #[pallet::call]
   impl <T:Config> Pallet<T>{
        #[pallet::weight(1_000)]
       // 创建存证，创建存证需要有两个关键参数：交易发送方origin，存证hash值claim，由于存证hash函数未知，也和decl_storage定义对应，这里使用变长Vec<u8>
        pub fn create_claim(
            origin:OriginFor<T>,
            proof:Vec<u8>,
        )-> DispatchResult{
            let sender = ensure_signed(origin)?;
            ensure!(!Proofs::<T>::contains_key(&proof),Error::<T>::ProofAlreadyClaimed);
            let current_block = <frame_system::Pallet<T>>::block_number();
            Proofs::<T>::insert(&proof,(&sender,current_block));
            Self::deposit_event(Event::ClaimCreated(sender,proof));
            Ok(())
        }
        //撤销存证
        #[pallet::weight(10_000)]
        pub fn revoke_claim(
            origin:OriginFor<T>,
            proof: Vec<u8>,
        )->DispatchResult{
            let sender = ensure_signed(origin)?;
            ensure!(Proofs::<T>::contains_key(&proof),Error::<T>::NoSuchProof);
            let (owner,_) = Proofs::<T>::get(&proof);
            ensure!(sender == owner,Error::<T>::NotProofOwner);

            Proofs::<T>::remove(&proof);
            Self::deposit_event(Event::ClaimRevoked(sender,proof));

            Ok(())
        }
        #[pallet::weight(10_000)]
        pub fn move_claim(
            origin:OriginFor<T>,
            proof:Vec<u8>,
        )->DispatchResult{
            let taker  = ensure_signed(origin)?;
            ensure!(Proofs::<T>::contains_key(&proof),Error::<T>::NoSuchProof);
            let (owner,_) = Proofs::<T>::get(&proof);
            //println!("taker {:?} owner {:?}",taker,owner);
            sp_std::if_std! {
                println!("taker {:?} owner {:?} is_true {:?}",taker,owner, taker==owner);
            }
            ensure!( !(taker == owner), Error::<T>::SameOwner);
            Proofs::<T>::remove(&proof); 
            let current_block = <frame_system::Pallet<T>>::block_number();
           // 以上校验完成之后，我们就可以删除我们的存证
		        // 存储向上调用remove函数进行删除
            Proofs::<T>::insert(&proof, (&taker,current_block));
            Self::deposit_event(Event::ClaimMoved(taker,proof)); 
            Ok(())
        }

   }

}

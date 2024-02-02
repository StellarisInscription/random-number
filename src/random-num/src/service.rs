use candid::{candid_method, Nat};
use ic_cdk::{api, init, query, update};
use ic_principal::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use num_bigint::BigUint;
use num_traits::ToBytes;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Debug)]
pub struct SeqNo(Nat);

impl Storable for SeqNo {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0 .0.to_le_bytes().to_vec().into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(Nat(BigUint::from_bytes_le(
            bytes.into_owned().to_vec().as_slice(),
        )))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 32,
        is_fixed_size: false,
    };
}

pub type RandomNo = SeqNo;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
      RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

  static OWNER: RefCell<StableCell<Principal, Memory>> = RefCell::new(
      StableCell::init(
          MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
          Principal::anonymous(),
      )
      .unwrap(),
  );

  static OPERATORS: RefCell<StableBTreeMap<Principal, (), Memory>> = RefCell::new(
      StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))),
  );

  static RANDOMS: RefCell<StableBTreeMap<SeqNo, RandomNo, Memory>> = RefCell::new(
      StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))),
  );
}

#[init]
fn init(owner: candid::Principal) {
    // if let Some(owner) = owner {
    //     let inner = Principal::from_slice(owner.as_slice());
    //     OWNER.with(|p| p.borrow_mut().set(inner).unwrap());
    // }
    let inner = Principal::from_slice(owner.as_slice());
        OWNER.with(|p| p.borrow_mut().set(inner).unwrap());
}

#[query]
#[candid_method(query)]
fn get_owner() -> candid::Principal {
    OWNER.with(|p| candid::Principal::from_slice(p.borrow().get().as_slice()))
}

#[update]
#[candid_method(update)]
fn add_operator(operator: candid::Principal) -> Result<bool, String> {
    // check if the caller is the owner
    let caller = api::caller();
    let owner = OWNER.with(|p| p.borrow().get().clone());
    if caller.as_slice() != owner.as_slice() {
        return Err(format!("Error: {:?} is not the owner", caller));
    }

    OPERATORS.with(|o| {
        o.borrow_mut()
            .insert(Principal::from_slice(operator.as_slice()), ())
    });

    Ok(true)
}

#[query]
#[candid_method(query)]
fn get_random_by_seq_no(seq_no: Nat) -> Option<Nat> {
    let exist = RANDOMS.with(|r| r.borrow().get(&SeqNo(seq_no.clone())));
    if exist.is_some() {
        return Some(exist.unwrap().0);
    }
    None
}

#[update]
#[candid_method(update)]
async fn generate_random(seq_no: Nat) -> Result<(Nat, Nat), String> {
    let operator = api::caller();
    if !validate_operator(Principal::from_slice(operator.as_slice())) {
        return Err(format!("Error: {:?} is not a valid operator", operator));
    }

    let exist = RANDOMS.with(|r| r.borrow().get(&SeqNo(seq_no.clone())));
    if exist.is_some() {
        return Ok((seq_no, exist.unwrap().0));
    }

    match api::management_canister::main::raw_rand().await {
        Ok((random,)) => {
            let random_no = SeqNo(BigUint::from_bytes_le(random.as_slice()).into());
            RANDOMS.with(|r| {
                r.borrow_mut()
                    .insert(SeqNo(seq_no.clone()), random_no.clone())
            });
            Ok((seq_no, random_no.0))
        }
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}

fn validate_operator(operator: Principal) -> bool {
    OPERATORS.with(|o| o.borrow().contains_key(&operator))
}

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
pub fn __export_did_tmp_() -> String {
    include_str!("../random_number.did").to_string()
}
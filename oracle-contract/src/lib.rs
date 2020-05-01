use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Default, BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
pub struct OracleRequest {
    caller: String,
    request_spec: String,
    token_address: String,
}

/* do this later: https://near-protocol.slack.com/archives/C010F7NT5QQ/p1587344292089800?thread_ts=1587342538.088000&cid=C010F7NT5QQ
#[derive(Default, BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
enum RevealType {
    Sha256,
    Keccak256,
    Xor
}
*/

/* do this later
impl Default for RevealType {
    fn default() -> Self { RevealType::Xor }
}
*/

#[derive(Default, BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
pub struct CommitReveal {
    commit_shrouded: String,
    reveal_answer: String,
    reveal_suffix: String,
    reveal_type: String
}

impl CommitReveal {
    pub fn new() -> Self {
        Self {
            commit_shrouded: String::new(),
            reveal_answer: String::new(),
            reveal_suffix: String::new(),
            reveal_type: String::new()
        }
    }
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
#[derive(Serialize)]
pub struct OracleRequests {
    // request id, struct
    reqs: HashMap<String, OracleRequest>,
    commitments_responses: HashMap<String, HashMap<String, CommitReveal>>
}

#[near_bindgen]
impl OracleRequests {
    #[init]
    // pub fn new(transfer_amount: U128, min_difficulty: u32) -> Self {
    pub fn new() -> Self {
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        Self {
            reqs: HashMap::new(),
            commitments_responses: HashMap::new()
        }
    }

    // TODO: remove caller, get from env
    pub fn add_request(&mut self, request_id: String, request_spec: String, token_address: String) {
        assert!(self.reqs.len() < 10, "Sorry, at the maximum length");
        assert!(!self.reqs.contains_key(&request_id), "Sorry, request_id already exists");
        env::log(b"We're adding another request");
        let oracle_req = OracleRequest {
            caller: env::signer_account_id(),
            request_spec: request_spec,
            token_address: token_address
        };
        self.reqs.insert(request_id, oracle_req);
    }

    #[payable]
    pub fn add_commitment(&mut self, request_id: String) {
        assert!(env::attached_deposit() > 19_000_000, "Please stake at least 19,000,000 yoctoⓃ");
        match self.commitments_responses.get_mut(&request_id) {
            Some(commitments) => {
                // request is already added to hashmap
                // check to see if this account has already committed
                assert!(!commitments.contains_key(  &env::signer_account_id()), "Already committed to this request");
                // TODO: move 3 (or max) somewhere else
                assert!(commitments.len() < 3, "Have max commitments");
                commitments.insert(env::signer_account_id(), CommitReveal::new());
            },
            None => {
                // request is not yet added to hashmap
                let mut commitment: HashMap<String, CommitReveal> = HashMap::new();
                commitment.insert(env::signer_account_id(), CommitReveal::new());
                self.commitments_responses.insert(request_id, commitment);
            }
        }
    }

    // TODO: add remove_request for an individual request

    // reveal_type is ["keccak256", "sha256", "xor"]
    pub fn add_answer_as_commit(&mut self, request_id: String, commit: String, reveal_type: String) {
        match self.commitments_responses.get_mut(&request_id) {
            Some(commitments) => {
                // request is in the hashmap
                // check to see if this account is on the list
                // assert!(commitments.contains_key(  &env::signer_account_id()), "This account is not on the list of committals for the request");
                match commitments.get_mut(&env::signer_account_id()) {
                    Some(commit_reveal) => {
                        commit_reveal.commit_shrouded = commit;
                        commit_reveal.reveal_type = reveal_type;
                    },
                    None => {
                        env::panic(b"This account is not on the list of committals for the request");
                    }
                }
                commitments.insert(env::signer_account_id(), CommitReveal::new());
            },
            None => {
                env::panic(b"No commitments for that request id");
            }
        }
    }

    pub fn clear_all_requests(&mut self) {
        // TODO: add "owner" var and check to make sure caller is the owner
        env::log(b"Clearing all requests");
        self.reqs.clear();
    }

    pub fn get_commitments_by_request_id(&self, request_id: String) -> String {
        env::log(b"Looking up commitments for request");

        match self.commitments_responses.get(&request_id) {
            Some(commitments) => {
                let serialized = serde_json::to_string(&commitments).unwrap();
                return serialized;
            },
            None => {
                env::panic(b"No commitments for that request id");
            }
        }
    }

    pub fn clear_all_commitments(&mut self) {
        // TODO: add "owner" var and check to make sure caller is the owner
        env::log(b"Clearing all commitments");
        self.commitments_responses.clear();
    }

    pub fn get_all_requests(&self) -> String {
        env::log(b"Returning all requests");
        let serialized = serde_json::to_string(&self.reqs).unwrap();
        return serialized;
    }

    pub fn get_all_commitments(&self) -> String {
        env::log(b"Returning all commitments");
        let serialized = serde_json::to_string(&self.commitments_responses).unwrap();
        return serialized;
    }

    // when an oracle is committing to helping with a request
    // TODO:
    // pub fn commit_to_request(&mut self, caller: String, request_id: u128, request_spec: String, token_address: String) {
    // }

    // TODO: store commit-reveal and mechanism to tell oracle we're ready for the reveal

    // TODO: add timeouts

    // TODO: add variance testing / aggregation
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn add_one_oracle_request() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = OracleRequests::default();
        contract.add_request("a".to_string(), "getLatestTokenPriceByHash".to_string(), "battokenaddy".to_string() );
        assert_eq!(1, contract.reqs.len());
    }

    #[test]
    fn add_and_remove_one_oracle_request() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = OracleRequests::default();
        contract.add_request("a".to_string(), "getLatestTokenPriceByHash".to_string(), "battokenaddy".to_string() );
        assert_eq!(1, contract.reqs.len());
        contract.clear_all_requests();
        assert_eq!(0, contract.reqs.len());
    }

    #[test]
    fn add_one_oracle_request_get_serialized() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = OracleRequests::default();
        contract.add_request("a".to_string(), "getLatestTokenPriceByHash".to_string(), "battokenaddy".to_string() );
        let serialized_output = contract.get_all_requests();
        assert_eq!("{\"a\":{\"caller\":\"bob_near\",\"request_spec\":\"getLatestTokenPriceByHash\",\"token_address\":\"battokenaddy\"}}", serialized_output);
    }

    #[test]
    fn add_two_oracle_requests_check_keys() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = OracleRequests::default();
        contract.add_request("request-id-alpha".to_string(), "getLatestTokenPriceByHash".to_string(), "battokenaddy".to_string() );
        contract.add_request("request-id-bravo".to_string(), "getLatestTokenPriceByHash".to_string(), "battokenaddy".to_string() );
        assert_eq!(true, contract.reqs.contains_key("request-id-alpha"));
        assert_eq!(true, contract.reqs.contains_key("request-id-bravo"));
        assert_eq!(false, contract.reqs.contains_key("request-id-charlie"));
    }
}
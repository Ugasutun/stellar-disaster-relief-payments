use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, String, Vec, Map, U256, u64};

#[contract]
pub struct AidRegistry;

#[derive(Clone)]
pub struct EmergencyFund {
    pub id: String,
    pub name: String,
    pub description: String,
    pub total_amount: U256,
    pub released_amount: U256,
    pub created_at: u64,
    pub expires_at: u64,
    pub disaster_type: String,
    pub geographic_scope: String,
    pub is_active: bool,
    pub release_triggers: Vec<Address>, // Multi-sig signers
    pub required_signatures: u32,
}

#[derive(Clone)]
pub struct DisbursementRecord {
    pub id: String,
    pub fund_id: String,
    pub beneficiary: Address,
    pub amount: U256,
    pub timestamp: u64,
    pub purpose: String,
    pub approved_by: Vec<Address>,
    pub transaction_hash: String,
}

#[contractimpl]
impl AidRegistry {
    /// Create a new emergency fund pool
    pub fn create_fund(
        env: Env,
        admin: Address,
        fund_id: String,
        name: String,
        description: String,
        total_amount: U256,
        disaster_type: String,
        geographic_scope: String,
        expires_at: u64,
        release_triggers: Vec<Address>,
        required_signatures: u32,
    ) {
        // Verify admin authorization
        admin.require_auth();
        
        // Create fund structure
        let fund = EmergencyFund {
            id: fund_id.clone(),
            name,
            description,
            total_amount,
            released_amount: U256::from_u64(0),
            created_at: env.ledger().timestamp(),
            expires_at,
            disaster_type,
            geographic_scope,
            is_active: true,
            release_triggers: release_triggers.clone(),
            required_signatures,
        };
        
        // Store fund
        let fund_key = Symbol::new(&env, "fund");
        let mut funds: Map<String, EmergencyFund> = env.storage().instance()
            .get(&fund_key)
            .unwrap_or(Map::new(&env));
        
        funds.set(fund_id.clone(), fund);
        env.storage().instance().set(&fund_key, &funds);
        
        // Initialize disbursement records for this fund
        let disbursement_key = Symbol::new(&env, &format!("disbursements_{}", fund_id));
        let disbursements: Map<String, DisbursementRecord> = Map::new(&env);
        env.storage().instance().set(&disbursement_key, &disbursements);
    }

    /// Get fund details
    pub fn get_fund(env: Env, fund_id: String) -> Option<EmergencyFund> {
        let fund_key = Symbol::new(&env, "fund");
        let funds: Map<String, EmergencyFund> = env.storage().instance()
            .get(&fund_key)
            .unwrap_or(Map::new(&env));
        
        funds.get(fund_id)
    }

    /// List all active funds
    pub fn list_active_funds(env: Env) -> Vec<EmergencyFund> {
        let fund_key = Symbol::new(&env, "fund");
        let funds: Map<String, EmergencyFund> = env.storage().instance()
            .get(&fund_key)
            .unwrap_or(Map::new(&env));
        
        let mut active_funds = Vec::new(&env);
        for (_, fund) in funds.iter() {
            if fund.is_active {
                active_funds.push_back(fund);
            }
        }
        active_funds
    }

    /// Submit disbursement request with multi-sig approval
    pub fn submit_disbursement(
        env: Env,
        requester: Address,
        fund_id: String,
        beneficiary: Address,
        amount: U256,
        purpose: String,
        approvers: Vec<Address>,
    ) {
        requester.require_auth();
        
        // Verify fund exists and is active
        let fund_key = Symbol::new(&env, "fund");
        let mut funds: Map<String, EmergencyFund> = env.storage().instance()
            .get(&fund_key)
            .unwrap_or(Map::new(&env));
        
        let mut fund = funds.get(fund_id.clone()).unwrap_or_panic_with(&env);
        
        if !fund.is_active {
            panic_with_error!(&env, "Fund is not active");
        }
        
        // Check if sufficient funds remain
        if fund.released_amount + amount > fund.total_amount {
            panic_with_error!(&env, "Insufficient funds in pool");
        }
        
        // Verify multi-sig requirements
        if approvers.len() < fund.required_signatures as usize {
            panic_with_error!(&env, "Insufficient signatures");
        }
        
        // Verify all approvers are authorized
        for approver in approvers.iter() {
            if !fund.release_triggers.contains(approver) {
                panic_with_error!(&env, "Unauthorized approver");
            }
        }
        
        // Create disbursement record
        let disbursement_id = format!("{}_{}", fund_id, env.ledger().timestamp());
        let disbursement = DisbursementRecord {
            id: disbursement_id.clone(),
            fund_id: fund_id.clone(),
            beneficiary,
            amount,
            timestamp: env.ledger().timestamp(),
            purpose,
            approved_by: approvers,
            transaction_hash: String::from_str(&env, ""), // Will be set after transaction
        };
        
        // Store disbursement
        let disbursement_key = Symbol::new(&env, &format!("disbursements_{}", fund_id));
        let mut disbursements: Map<String, DisbursementRecord> = env.storage().instance()
            .get(&disbursement_key)
            .unwrap_or(Map::new(&env));
        
        disbursements.set(disbursement_id.clone(), disbursement);
        env.storage().instance().set(&disbursement_key, &disbursements);
        
        // Update fund released amount
        fund.released_amount += amount;
        funds.set(fund_id, fund);
        env.storage().instance().set(&fund_key, &funds);
    }

    /// Get disbursement history for a fund
    pub fn get_disbursements(env: Env, fund_id: String) -> Vec<DisbursementRecord> {
        let disbursement_key = Symbol::new(&env, &format!("disbursements_{}", fund_id));
        let disbursements: Map<String, DisbursementRecord> = env.storage().instance()
            .get(&disbursement_key)
            .unwrap_or(Map::new(&env));
        
        let mut result = Vec::new(&env);
        for (_, record) in disbursements.iter() {
            result.push_back(record);
        }
        result
    }

    /// Deactivate expired funds
    pub fn cleanup_expired_funds(env: Env) {
        let fund_key = Symbol::new(&env, "fund");
        let mut funds: Map<String, EmergencyFund> = env.storage().instance()
            .get(&fund_key)
            .unwrap_or(Map::new(&env));
        
        let current_time = env.ledger().timestamp();
        
        for (fund_id, mut fund) in funds.iter() {
            if current_time > fund.expires_at && fund.is_active {
                fund.is_active = false;
                funds.set(fund_id, fund);
            }
        }
        
        env.storage().instance().set(&fund_key, &funds);
    }
}

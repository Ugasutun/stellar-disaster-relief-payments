use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, String, Vec, Map, U256, u64};

#[contract]
pub struct MerchantNetwork;

#[derive(Clone)]
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub owner: Address,
    pub business_type: String,
    pub location: Location,
    pub contact_info: String,
    pub registration_date: u64,
    pub is_verified: bool,
    pub verification_documents: Vec<String>,
    pub stellar_toml_url: String,
    pub accepted_tokens: Vec<String>,
    pub daily_limit: U256,
    pub monthly_limit: U256,
    pub current_month_volume: U256,
    pub reputation_score: u32, // 0-100
    pub is_active: bool,
}

#[derive(Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub city: String,
    pub country: String,
    pub postal_code: String,
}

#[derive(Clone)]
pub struct Transaction {
    pub id: String,
    pub merchant_id: String,
    pub beneficiary_id: String,
    pub amount: U256,
    pub token: String,
    pub timestamp: u64,
    pub purpose: String,
    pub merchant_signature: String,
    pub beneficiary_signature: String,
    pub is_settled: bool,
}

#[contractimpl]
impl MerchantNetwork {
    /// Register a local merchant for the relief network
    pub fn register_merchant(
        env: Env,
        owner: Address,
        merchant_id: String,
        name: String,
        business_type: String,
        location: Location,
        contact_info: String,
        stellar_toml_url: String,
        accepted_tokens: Vec<String>,
        daily_limit: U256,
        monthly_limit: U256,
        verification_documents: Vec<String>,
    ) {
        owner.require_auth();
        
        // Check for duplicate registration
        let merchants_key = Symbol::new(&env, "merchants");
        let merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        if merchants.contains_key(merchant_id.clone()) {
            panic_with_error!(&env, "Merchant already registered");
        }
        
        // Create merchant profile
        let merchant = Merchant {
            id: merchant_id.clone(),
            name,
            owner,
            business_type,
            location,
            contact_info,
            registration_date: env.ledger().timestamp(),
            is_verified: false, // Requires manual verification
            verification_documents,
            stellar_toml_url,
            accepted_tokens,
            daily_limit,
            monthly_limit,
            current_month_volume: U256::from_u64(0),
            reputation_score: 50, // Initial reputation
            is_active: false, // Activated after verification
        };
        
        merchants.set(merchant_id.clone(), merchant);
        env.storage().instance().set(&merchants_key, &merchants);
        
        // Add to verification queue
        let verification_queue_key = Symbol::new(&env, "verification_queue");
        let mut queue: Vec<String> = env.storage().instance()
            .get(&verification_queue_key)
            .unwrap_or(Vec::new(&env));
        
        queue.push_back(merchant_id);
        env.storage().instance().set(&verification_queue_key, &queue);
    }

    /// Verify merchant (by authorized verifier)
    pub fn verify_merchant(
        env: Env,
        verifier: Address,
        merchant_id: String,
        approved: bool,
        notes: String,
    ) {
        verifier.require_auth();
        
        let merchants_key = Symbol::new(&env, "merchants");
        let mut merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut merchant) = merchants.get(merchant_id.clone()) {
            if approved {
                merchant.is_verified = true;
                merchant.is_active = true;
            }
            merchants.set(merchant_id, merchant);
            env.storage().instance().set(&merchants_key, &merchants);
        }
        
        // Remove from verification queue
        let verification_queue_key = Symbol::new(&env, "verification_queue");
        let mut queue: Vec<String> = env.storage().instance()
            .get(&verification_queue_key)
            .unwrap_or(Vec::new(&env));
        
        let mut new_queue = Vec::new(&env);
        for id in queue.iter() {
            if id != merchant_id {
                new_queue.push_back(id);
            }
        }
        env.storage().instance().set(&verification_queue_key, &new_queue);
    }

    /// Process payment from beneficiary to merchant
    pub fn process_payment(
        env: Env,
        merchant: Address,
        beneficiary: Address,
        merchant_id: String,
        beneficiary_id: String,
        amount: U256,
        token: String,
        purpose: String,
    ) -> String {
        merchant.require_auth();
        beneficiary.require_auth();
        
        // Verify merchant exists and is active
        let merchants_key = Symbol::new(&env, "merchants");
        let mut merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        let mut merchant_profile = match merchants.get(merchant_id.clone()) {
            Some(m) => m,
            None => panic_with_error!(&env, "Merchant not found"),
        };
        
        if !merchant_profile.is_active {
            panic_with_error!(&env, "Merchant is not active");
        }
        
        // Check if token is accepted
        if !merchant_profile.accepted_tokens.contains(&token) {
            panic_with_error!(&env, "Token not accepted by merchant");
        }
        
        // Check limits (simplified - in production, implement proper time-based limits)
        if amount > merchant_profile.daily_limit {
            panic_with_error!(&env, "Amount exceeds daily limit");
        }
        
        if merchant_profile.current_month_volume + amount > merchant_profile.monthly_limit {
            panic_with_error!(&env, "Amount exceeds monthly limit");
        }
        
        // Create transaction record
        let transaction_id = format!("tx_{}_{}", merchant_id, env.ledger().timestamp());
        let transaction = Transaction {
            id: transaction_id.clone(),
            merchant_id: merchant_id.clone(),
            beneficiary_id,
            amount,
            token,
            timestamp: env.ledger().timestamp(),
            purpose,
            merchant_signature: String::from_str(&env, "merchant_signed"), // In production, actual signatures
            beneficiary_signature: String::from_str(&env, "beneficiary_signed"),
            is_settled: false,
        };
        
        // Store transaction
        let transactions_key = Symbol::new(&env, "transactions");
        let mut transactions: Map<String, Transaction> = env.storage().instance()
            .get(&transactions_key)
            .unwrap_or(Map::new(&env));
        
        transactions.set(transaction_id.clone(), transaction);
        env.storage().instance().set(&transactions_key, &transactions);
        
        // Update merchant volume
        merchant_profile.current_month_volume += amount;
        merchants.set(merchant_id, merchant_profile);
        env.storage().instance().set(&merchants_key, &merchants);
        
        transaction_id
    }

    /// Get merchant details
    pub fn get_merchant(env: Env, merchant_id: String) -> Option<Merchant> {
        let merchants_key = Symbol::new(&env, "merchants");
        let merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        merchants.get(merchant_id)
    }

    /// Find merchants by location (geographic search)
    pub fn find_merchants_by_location(
        env: Env,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Vec<Merchant> {
        let merchants_key = Symbol::new(&env, "merchants");
        let merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        let mut nearby_merchants = Vec::new(&env);
        
        for (_, merchant) in merchants.iter() {
            if merchant.is_active {
                let distance = Self::calculate_distance(
                    latitude, longitude,
                    merchant.location.latitude, merchant.location.longitude
                );
                
                if distance <= radius_km {
                    nearby_merchants.push_back(merchant);
                }
            }
        }
        
        nearby_merchants
    }

    /// Simple distance calculation (Haversine formula approximation)
    fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0; // Earth's radius in km
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2) +
                lat1.to_radians().cos() * lat2.to_radians().cos() *
                (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    /// Get merchant transaction history
    pub fn get_merchant_transactions(env: Env, merchant_id: String) -> Vec<Transaction> {
        let transactions_key = Symbol::new(&env, "transactions");
        let transactions: Map<String, Transaction> = env.storage().instance()
            .get(&transactions_key)
            .unwrap_or(Map::new(&env));
        
        let mut merchant_transactions = Vec::new(&env);
        for (_, transaction) in transactions.iter() {
            if transaction.merchant_id == merchant_id {
                merchant_transactions.push_back(transaction);
            }
        }
        merchant_transactions
    }

    /// Update merchant reputation based on feedback
    pub fn update_reputation(
        env: Env,
        admin: Address,
        merchant_id: String,
        feedback_score: i32, // -10 to +10
    ) {
        admin.require_auth();
        
        let merchants_key = Symbol::new(&env, "merchants");
        let mut merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut merchant) = merchants.get(merchant_id.clone()) {
            let new_score = (merchant.reputation_score as i32 + feedback_score).max(0).min(100);
            merchant.reputation_score = new_score as u32;
            merchants.set(merchant_id, merchant);
            env.storage().instance().set(&merchants_key, &merchants);
        }
    }

    /// Reset monthly volume (called periodically)
    pub fn reset_monthly_volumes(env: Env) {
        let merchants_key = Symbol::new(&env, "merchants");
        let mut merchants: Map<String, Merchant> = env.storage().instance()
            .get(&merchants_key)
            .unwrap_or(Map::new(&env));
        
        for (merchant_id, mut merchant) in merchants.iter() {
            merchant.current_month_volume = U256::from_u64(0);
            merchants.set(merchant_id, merchant);
        }
        
        env.storage().instance().set(&merchants_key, &merchants);
    }

    /// Get verification queue
    pub fn get_verification_queue(env: Env) -> Vec<String> {
        let verification_queue_key = Symbol::new(&env, "verification_queue");
        env.storage().instance()
            .get(&verification_queue_key)
            .unwrap_or(Vec::new(&env))
    }
}

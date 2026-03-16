use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, String, Vec, Map, U256, u64, BytesN};

#[contract]
pub struct BeneficiaryManager;

#[derive(Clone)]
pub struct BeneficiaryProfile {
    pub id: String,
    pub name: String,
    pub disaster_id: String,
    pub location: String,
    pub registration_date: u64,
    pub last_verified: u64,
    pub verification_factors: Vec<VerificationFactor>,
    pub wallet_address: Address,
    pub is_active: bool,
    pub family_size: u32,
    pub special_needs: Vec<String>,
    pub trust_score: u32, // 0-100 based on behavioral patterns
}

#[derive(Clone)]
pub struct VerificationFactor {
    pub factor_type: String, // "possession", "behavioral", "social"
    pub value: String,
    pub weight: u32,
    pub verified_at: u64,
}

#[derive(Clone)]
pub struct RecoveryCode {
    pub beneficiary_id: String,
    pub code_hash: BytesN<32>,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_used: bool,
}

#[contractimpl]
impl BeneficiaryManager {
    /// Register a displaced person without traditional ID
    pub fn register_beneficiary(
        env: Env,
        registrar: Address,
        beneficiary_id: String,
        name: String,
        disaster_id: String,
        location: String,
        wallet_address: Address,
        family_size: u32,
        special_needs: Vec<String>,
        verification_factors: Vec<VerificationFactor>,
    ) {
        registrar.require_auth();
        
        // Check for duplicate registrations
        let beneficiaries_key = Symbol::new(&env, "beneficiaries");
        let beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
            .get(&beneficiaries_key)
            .unwrap_or(Map::new(&env));
        
        if beneficiaries.contains_key(beneficiary_id.clone()) {
            panic_with_error!(&env, "Beneficiary already registered");
        }
        
        // Create beneficiary profile
        let profile = BeneficiaryProfile {
            id: beneficiary_id.clone(),
            name,
            disaster_id,
            location,
            registration_date: env.ledger().timestamp(),
            last_verified: env.ledger().timestamp(),
            verification_factors,
            wallet_address,
            is_active: true,
            family_size,
            special_needs,
            trust_score: 50, // Initial trust score
        };
        
        beneficiaries.set(beneficiary_id.clone(), profile);
        env.storage().instance().set(&beneficiaries_key, &beneficiaries);
        
        // Generate recovery codes for account restoration
        Self::generate_recovery_codes(&env, beneficiary_id);
    }

    /// Generate recovery codes for offline access restoration
    fn generate_recovery_codes(env: &Env, beneficiary_id: String) {
        let recovery_key = Symbol::new(env, "recovery_codes");
        let mut recovery_codes: Map<String, Vec<RecoveryCode>> = env.storage().instance()
            .get(&recovery_key)
            .unwrap_or(Map::new(env));
        
        let mut codes = Vec::new(env);
        let current_time = env.ledger().timestamp();
        
        // Generate 3 recovery codes with different expiry times
        for i in 0..3 {
            let code_hash = Self::hash_recovery_code(env, &beneficiary_id, i);
            let recovery_code = RecoveryCode {
                beneficiary_id: beneficiary_id.clone(),
                code_hash,
                created_at: current_time,
                expires_at: current_time + (86400 * (i + 1) * 30), // 30, 60, 90 days
                is_used: false,
            };
            codes.push_back(recovery_code);
        }
        
        recovery_codes.set(beneficiary_id, codes);
        env.storage().instance().set(&recovery_key, &recovery_codes);
    }

    /// Simple hash function for recovery codes (in production, use secure hashing)
    fn hash_recovery_code(env: &Env, beneficiary_id: &String, index: i32) -> BytesN<32> {
        use soroban_sdk::crypto::sha256;
        let mut data = Vec::new(env);
        data.push_back(String::from_str(env, beneficiary_id));
        data.push_back(String::from_str(env, &index.to_string()));
        sha256(&data.to_string().into())
    }

    /// Verify beneficiary using behavioral/possession factors
    pub fn verify_beneficiary(
        env: Env,
        verifier: Address,
        beneficiary_id: String,
        provided_factors: Vec<VerificationFactor>,
    ) -> bool {
        verifier.require_auth();
        
        let beneficiaries_key = Symbol::new(&env, "beneficiaries");
        let mut beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
            .get(&beneficiaries_key)
            .unwrap_or(Map::new(&env));
        
        let mut profile = match beneficiaries.get(beneficiary_id.clone()) {
            Some(p) => p,
            None => return false,
        };
        
        // Calculate verification score
        let mut total_weight = 0u32;
        let mut matched_weight = 0u32;
        
        for stored_factor in profile.verification_factors.iter() {
            total_weight += stored_factor.weight;
            
            for provided_factor in provided_factors.iter() {
                if stored_factor.factor_type == provided_factor.factor_type 
                    && stored_factor.value == provided_factor.value {
                    matched_weight += stored_factor.weight;
                    break;
                }
            }
        }
        
        let verification_score = if total_weight > 0 {
            (matched_weight * 100) / total_weight
        } else {
            0
        };
        
        // Update trust score based on verification success
        if verification_score >= 70 {
            profile.trust_score = (profile.trust_score + 10).min(100);
            profile.last_verified = env.ledger().timestamp();
            beneficiaries.set(beneficiary_id, profile);
            env.storage().instance().set(&beneficiaries_key, &beneficiaries);
            true
        } else {
            profile.trust_score = profile.trust_score.saturating_sub(5);
            beneficiaries.set(beneficiary_id, profile);
            env.storage().instance().set(&beneficiaries_key, &beneficiaries);
            false
        }
    }

    /// Restore access using recovery code
    pub fn restore_access(
        env: Env,
        beneficiary_id: String,
        recovery_code: BytesN<32>,
        new_wallet: Address,
    ) -> bool {
        let recovery_key = Symbol::new(&env, "recovery_codes");
        let mut recovery_codes: Map<String, Vec<RecoveryCode>> = env.storage().instance()
            .get(&recovery_key)
            .unwrap_or(Map::new(&env));
        
        let current_time = env.ledger().timestamp();
        
        if let Some(mut codes) = recovery_codes.get(beneficiary_id.clone()) {
            for mut code in codes.iter() {
                if code.code_hash == recovery_code 
                    && !code.is_used 
                    && current_time <= code.expires_at {
                    
                    // Mark code as used
                    code.is_used = true;
                    
                    // Update beneficiary wallet address
                    let beneficiaries_key = Symbol::new(&env, "beneficiaries");
                    let mut beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
                        .get(&beneficiaries_key)
                        .unwrap_or(Map::new(&env));
                    
                    if let Some(mut profile) = beneficiaries.get(beneficiary_id.clone()) {
                        profile.wallet_address = new_wallet;
                        profile.last_verified = current_time;
                        beneficiaries.set(beneficiary_id, profile);
                        env.storage().instance().set(&beneficiaries_key, &beneficiaries);
                    }
                    
                    return true;
                }
            }
        }
        
        false
    }

    /// Get beneficiary profile
    pub fn get_beneficiary(env: Env, beneficiary_id: String) -> Option<BeneficiaryProfile> {
        let beneficiaries_key = Symbol::new(&env, "beneficiaries");
        let beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
            .get(&beneficiaries_key)
            .unwrap_or(Map::new(&env));
        
        beneficiaries.get(beneficiary_id)
    }

    /// List beneficiaries by disaster
    pub fn list_beneficiaries_by_disaster(env: Env, disaster_id: String) -> Vec<BeneficiaryProfile> {
        let beneficiaries_key = Symbol::new(&env, "beneficiaries");
        let beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
            .get(&beneficiaries_key)
            .unwrap_or(Map::new(&env));
        
        let mut result = Vec::new(&env);
        for (_, profile) in beneficiaries.iter() {
            if profile.disaster_id == disaster_id && profile.is_active {
                result.push_back(profile);
            }
        }
        result
    }

    /// Update beneficiary location (for tracking displacement)
    pub fn update_location(
        env: Env,
        beneficiary: Address,
        beneficiary_id: String,
        new_location: String,
    ) {
        beneficiary.require_auth();
        
        let beneficiaries_key = Symbol::new(&env, "beneficiaries");
        let mut beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
            .get(&beneficiaries_key)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut profile) = beneficiaries.get(beneficiary_id.clone()) {
            profile.location = new_location;
            profile.last_verified = env.ledger().timestamp();
            beneficiaries.set(beneficiary_id, profile);
            env.storage().instance().set(&beneficiaries_key, &beneficiaries);
        }
    }

    /// Deactivate beneficiary (e.g., when they leave the program)
    pub fn deactivate_beneficiary(env: Env, admin: Address, beneficiary_id: String) {
        admin.require_auth();
        
        let beneficiaries_key = Symbol::new(&env, "beneficiaries");
        let mut beneficiaries: Map<String, BeneficiaryProfile> = env.storage().instance()
            .get(&beneficiaries_key)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut profile) = beneficiaries.get(beneficiary_id.clone()) {
            profile.is_active = false;
            beneficiaries.set(beneficiary_id, profile);
            env.storage().instance().set(&beneficiaries_key, &beneficiaries);
        }
    }
}

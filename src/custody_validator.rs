use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, String, Vec, Map, U256, u64};

// Asset verification types
pub const ASSET_TYPE_REAL_ESTATE: u32 = 0;
pub const ASSET_TYPE_PRECIOUS_METALS: u32 = 1;
pub const ASSET_TYPE_ART: u32 = 2;
pub const ASSET_TYPE_COMMODITIES: u32 = 3;
pub const ASSET_TYPE_INVOICE: u32 = 4;

// Custodian status
pub const CUSTODIAN_STATUS_PENDING: u32 = 0;
pub const CUSTODIAN_STATUS_ACTIVE: u32 = 1;
pub const CUSTODIAN_STATUS_SUSPENDED: u32 = 2;
pub const CUSTODIAN_STATUS_REVOKED: u32 = 3;

// Attestation status
pub const ATTESTATION_STATUS_PENDING: u32 = 0;
pub const ATTESTATION_STATUS_VERIFIED: u32 = 1;
pub const ATTESTATION_STATUS_DISPUTED: u32 = 2;
pub const ATTESTATION_STATUS_EXPIRED: u32 = 3;
pub const ATTESTATION_STATUS_SLASHED: u32 = 4;

// Verification thresholds
pub const MULTISIG_THRESHOLD: u32 = 3; // 3-of-5 verification required
pub const MAX_CUSTODIANS: u32 = 50;
pub const DISPUTE_BOND_AMOUNT: U256 = U256::from_u32(10000); // 10000 tokens bond

#[contract]
pub struct CustodyValidator;

#[derive(Clone)]
pub struct CustodyAttestation {
    pub asset_id: String,
    pub custodian: Address,
    pub location: String,
    pub condition: String,
    pub value: U256,
    pub timestamp: u64,
    pub proof_hash: String,
    pub asset_type: u32,
    pub status: u32,
    pub verification_count: u32,
    pub signatures: Vec<Address>,
}

#[derive(Clone)]
pub struct CustodianInfo {
    pub name: String,
    pub jurisdiction: String,
    pub license: String,
    pub reputation_score: u32,
    pub status: u32,
    pub registered_date: u64,
    pub total_attestations: u32,
    pub successful_verifications: u32,
    pub failed_verifications: u32,
}

#[derive(Clone)]
pub struct CustodyAlert {
    pub asset_id: String,
    pub alert_type: u32,
    pub message: String,
    pub timestamp: u64,
    pub severity: u32,
    pub resolved: bool,
}

#[derive(Clone)]
pub struct Dispute {
    pub dispute_id: String,
    pub asset_id: String,
    pub challenger: Address,
    pub reason: String,
    pub bond_amount: U256,
    pub timestamp: u64,
    pub status: u32,
    pub resolution: String,
    pub slashed: bool,
}

// Alert types
pub const ALERT_EXPIRED_ATTESTATION: u32 = 0;
pub const ALERT_MISSING_ATTESTATION: u32 = 1;
pub const ALERT_CUSTODIAN_SUSPENDED: u32 = 2;
pub const ALERT_VALUE_DEVIATION: u32 = 3;
pub const ALERT_INSURANCE_EXPIRED: u32 = 4;

// Severity levels
pub const SEVERITY_LOW: u32 = 0;
pub const SEVERITY_MEDIUM: u32 = 1;
pub const SEVERITY_HIGH: u32 = 2;
pub const SEVERITY_CRITICAL: u32 = 3;

// Dispute status
pub const DISPUTE_STATUS_OPEN: u32 = 0;
pub const DISPUTE_STATUS_RESOLVED: u32 = 1;
pub const DISPUTE_STATUS_REJECTED: u32 = 2;
pub const DISPUTE_STATUS_SLASHED: u32 = 3;

#[contractimpl]
impl CustodyValidator {
    /// Initialize the custody validator with admin
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&Symbol::new(&env, "admin"), &admin);
        env.storage().instance().set(&Symbol::new(&env, "initialized"), &true);
        env.storage().instance().set(&Symbol::new(&env, "custodian_count"), &0u32);
        env.storage().instance().set(&Symbol::new(&env, "attestation_count"), &0u64);
    }

    /// Register a new custodian (whitelist)
    pub fn register_custodian(
        env: Env,
        caller: Address,
        name: String,
        jurisdiction: String,
        license: String,
    ) -> Result<u32, String> {
        // Verify admin
        let admin: Address = env.storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .ok_or("Admin not set")?;
        
        if caller != admin {
            return Err("Only admin can register custodians".to_string());
        }

        // Check max custodians
        let count: u32 = env.storage()
            .instance()
            .get(&Symbol::new(&env, "custodian_count"))
            .unwrap_or(0);

        if count >= MAX_CUSTODIANS {
            return Err("Maximum custodians reached".to_string());
        }

        let custodian_id = count + 1;
        let custodian = CustodianInfo {
            name,
            jurisdiction,
            license,
            reputation_score: 100, // Start with perfect reputation
            status: CUSTODIAN_STATUS_ACTIVE,
            registered_date: env.ledger().timestamp(),
            total_attestations: 0,
            successful_verifications: 0,
            failed_verifications: 0,
        };

        // Store custodian
        let custodian_key = Symbol::new(&env, "custodian");
        let mut custodians: Map<u32, CustodianInfo> = env.storage()
            .instance()
            .get(&custodian_key)
            .unwrap_or(Map::new(&env));
        
        custodians.set(custodian_id, custodian.clone());
        env.storage().instance().set(&custodian_key, &custodians);
        env.storage().instance().set(&Symbol::new(&env, "custodian_count"), &custodian_id);

        Ok(custodian_id)
    }

    /// Submit an attestation for an asset
    pub fn submit_attestation(
        env: Env,
        caller: Address,
        custodian_id: u32,
        asset_id: String,
        location: String,
        condition: String,
        value: U256,
        proof_hash: String,
        asset_type: u32,
    ) -> Result<u64, String> {
        // Verify custodian is registered and active
        let custodian_key = Symbol::new(&env, "custodian");
        let custodians: Map<u32, CustodianInfo> = env.storage()
            .instance()
            .get(&custodian_key)
            .ok_or("Custodians not initialized")?;

        let custodian = custodians.get(custodian_id).ok_or("Custodian not found")?;
        
        if custodian.status != CUSTODIAN_STATUS_ACTIVE {
            return Err("Custodian is not active".to_string());
        }

        // Generate attestation ID
        let attestation_count: u64 = env.storage()
            .instance()
            .get(&Symbol::new(&env, "attestation_count"))
            .unwrap_or(0);
        
        let attestation_id = attestation_count + 1;

        let attestation = CustodyAttestation {
            asset_id: asset_id.clone(),
            custodian: caller,
            location,
            condition,
            value,
            timestamp: env.ledger().timestamp(),
            proof_hash,
            asset_type,
            status: ATTESTATION_STATUS_PENDING,
            verification_count: 0,
            signatures: {
                let mut sigs = Vec::new(&env);
                sigs.push_back(caller);
                sigs
            },
        };

        // Store attestation
        let attestation_key = Symbol::new(&env, "attestation");
        let mut attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .unwrap_or(Map::new(&env));
        
        attestations.set(attestation_id, attestation);
        env.storage().instance().set(&attestation_key, &attestations);
        env.storage().instance().set(&Symbol::new(&env, "attestation_count"), &attestation_id);

        // Update asset to attestation mapping
        let asset_key = Symbol::new(&env, "asset_atts");
        let mut asset_atts: Map<String, Vec<u64>> = env.storage()
            .instance()
            .get(&asset_key)
            .unwrap_or(Map::new(&env));
        
        let mut att_ids = asset_atts.get(asset_id.clone()).unwrap_or(Vec::new(&env));
        att_ids.push_back(attestation_id);
        asset_atts.set(asset_id, att_ids);
        env.storage().instance().set(&asset_key, &asset_atts);

        // Update custodian stats
        let mut updated_custodians = custodians;
        let mut updated_custodian = custodian;
        updated_custodian.total_attestations += 1;
        updated_custodians.set(custodian_id, updated_custodian);
        env.storage().instance().set(&custodian_key, &updated_custodians);

        Ok(attestation_id)
    }

    /// Verify an attestation with multi-sig (3-of-5)
    pub fn verify_attestation(
        env: Env,
        caller: Address,
        attestation_id: u64,
    ) -> Result<bool, String> {
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        let mut attestation = attestations.get(attestation_id).ok_or("Attestation not found")?;
        
        if attestation.status == ATTESTATION_STATUS_VERIFIED {
            return Err("Attestation already verified".to_string());
        }

        if attestation.status == ATTESTATION_STATUS_DISPUTED {
            return Err("Attestation is under dispute".to_string());
        }

        // Add signature
        let mut signatures = attestation.signatures;
        
        // Check if already signed
        let mut already_signed = false;
        for i in 0..signatures.len() {
            if signatures.get(i).unwrap() == caller {
                already_signed = true;
                break;
            }
        }
        
        if !already_signed {
            signatures.push_back(caller);
        }
        
        attestation.signatures = signatures;
        attestation.verification_count += 1;

        // Check if threshold reached (3-of-5)
        if attestation.verification_count >= MULTISIG_THRESHOLD {
            attestation.status = ATTESTATION_STATUS_VERIFIED;
            
            // Update custodian reputation
            let custodian_key = Symbol::new(&env, "custodian");
            let custodians: Map<u32, CustodianInfo> = env.storage()
                .instance()
                .get(&custodian_key)
                .ok_or("Custodians not initialized")?;
            
            // Find custodian by address
            let mut updated_custodians = custodians;
            for (id, mut cust) in custodians.iter() {
                if cust.successful_verifications > 0 || cust.failed_verifications > 0 {
                    // Update successful verifications
                    cust.successful_verifications += 1;
                    // Update reputation score
                    let total = cust.successful_verifications + cust.failed_verifications;
                    cust.reputation_score = (cust.successful_verifications * 100) / total;
                    updated_custodians.set(id, cust);
                }
            }
            env.storage().instance().set(&custodian_key, &updated_custodians);
        }

        // Update attestation
        let mut updated_attestations = attestations;
        updated_attestations.set(attestation_id, attestation);
        env.storage().instance().set(&attestation_key, &updated_attestations);

        Ok(attestation.verification_count >= MULTISIG_THRESHOLD)
    }

    /// Dispute an attestation with bond staking
    pub fn dispute_attestation(
        env: Env,
        caller: Address,
        asset_id: String,
        reason: String,
        bond: U256,
    ) -> Result<String, String> {
        // Verify bond amount
        if bond < DISPUTE_BOND_AMOUNT {
            return Err("Insufficient bond amount".to_string());
        }

        // Find latest attestation for asset
        let asset_key = Symbol::new(&env, "asset_atts");
        let asset_atts: Map<String, Vec<u64>> = env.storage()
            .instance()
            .get(&asset_key)
            .ok_or("No attestations for asset")?;

        let att_ids = asset_atts.get(asset_id.clone()).ok_or("Asset not found")?;
        
        if att_ids.len() == 0 {
            return Err("No attestations found".to_string());
        }

        // Get latest attestation
        let latest_att_id = att_ids.get(att_ids.len() - 1).ok_or("Invalid attestation")?;
        
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        let mut attestation = attestations.get(latest_att_id).ok_or("Attestation not found")?;
        
        // Mark attestation as disputed
        attestation.status = ATTESTATION_STATUS_DISPUTED;
        
        let mut updated_attestations = attestations;
        updated_attestations.set(latest_att_id, attestation);
        env.storage().instance().set(&attestation_key, &updated_attestations);

        // Create dispute record
        let dispute_id = format!("DISP-{}", latest_att_id);
        let dispute = Dispute {
            dispute_id: dispute_id.clone(),
            asset_id,
            challenger: caller,
            reason,
            bond_amount: bond,
            timestamp: env.ledger().timestamp(),
            status: DISPUTE_STATUS_OPEN,
            resolution: String::from("Pending resolution"),
            slashed: false,
        };

        // Store dispute
        let dispute_key = Symbol::new(&env, "dispute");
        let mut disputes: Map<String, Dispute> = env.storage()
            .instance()
            .get(&dispute_key)
            .unwrap_or(Map::new(&env));
        
        disputes.set(dispute_id.clone(), dispute);
        env.storage().instance().set(&dispute_key, &disputes);

        Ok(dispute_id)
    }

    /// Resolve a dispute (admin function)
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        dispute_id: String,
        resolution: String,
        slash_custodian: bool,
    ) -> Result<bool, String> {
        // Verify admin
        let admin: Address = env.storage()
            .instance()
            .get(&Symbol::new(&env, "admin"))
            .ok_or("Admin not set")?;
        
        if caller != admin {
            return Err("Only admin can resolve disputes".to_string());
        }

        let dispute_key = Symbol::new(&env, "dispute");
        let disputes: Map<String, Dispute> = env.storage()
            .instance()
            .get(&dispute_key)
            .ok_or("Disputes not initialized")?;

        let mut dispute = disputes.get(dispute_id.clone()).ok_or("Dispute not found")?;
        
        dispute.status = if slash_custodian {
            DISPUTE_STATUS_SLASHED
        } else {
            DISPUTE_STATUS_RESOLVED
        };
        dispute.resolution = resolution;
        dispute.slashed = slash_custodian;

        // Update attestation status
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        // Parse attestation ID from dispute
        let att_id_str = dispute_id.replace("DISP-", "");
        let att_id: u64 = att_id_str.parse().unwrap_or(0);
        
        if att_id > 0 {
            let mut updated_attestations = attestations;
            if let Some(mut attestation) = updated_attestations.get(att_id) {
                if slash_custodian {
                    attestation.status = ATTESTATION_STATUS_SLASHED;
                } else {
                    attestation.status = ATTESTATION_STATUS_VERIFIED;
                }
                updated_attestations.set(att_id, attestation);
                env.storage().instance().set(&attestation_key, &updated_attestations);
            }
        }

        // Update dispute
        let mut updated_disputes = disputes;
        updated_disputes.set(dispute_id, dispute);
        env.storage().instance().set(&dispute_key, &updated_disputes);

        Ok(slash_custodian)
    }

    /// Check if all attestations for an asset are current and valid
    pub fn verify_asset_backing(env: Env, asset_id: String) -> Result<bool, String> {
        let asset_key = Symbol::new(&env, "asset_atts");
        let asset_atts: Map<String, Vec<u64>> = env.storage()
            .instance()
            .get(&asset_key)
            .ok_or("No attestations for asset")?;

        let att_ids = asset_atts.get(asset_id.clone()).ok_or("Asset not found")?;
        
        if att_ids.len() == 0 {
            return Err("No attestations found".to_string());
        }

        // Get latest attestation
        let latest_att_id = att_ids.get(att_ids.len() - 1).ok_or("Invalid attestation")?;
        
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        let attestation = attestations.get(latest_att_id).ok_or("Attestation not found")?;
        
        // Check if verified and not expired/disputed
        Ok(attestation.status == ATTESTATION_STATUS_VERIFIED)
    }

    /// Get custody history for an asset
    pub fn get_custody_history(
        env: Env,
        asset_id: String,
    ) -> Result<Vec<CustodyAttestation>, String> {
        let asset_key = Symbol::new(&env, "asset_atts");
        let asset_atts: Map<String, Vec<u64>> = env.storage()
            .instance()
            .get(&asset_key)
            .ok_or("No attestations for asset")?;

        let att_ids = asset_atts.get(asset_id.clone()).ok_or("Asset not found")?;
        
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        let mut history = Vec::new(&env);
        
        for i in 0..att_ids.len() {
            let att_id = att_ids.get(i).unwrap();
            if let Some(att) = attestations.get(att_id) {
                history.push_back(att);
            }
        }

        Ok(history)
    }

    /// Get custodian info
    pub fn get_custodian(env: Env, custodian_id: u32) -> Result<CustodianInfo, String> {
        let custodian_key = Symbol::new(&env, "custodian");
        let custodians: Map<u32, CustodianInfo> = env.storage()
            .instance()
            .get(&custodian_key)
            .ok_or("Custodians not initialized")?;

        custodians.get(custodian_id).ok_or("Custodian not found")
    }

    /// Get all active custodians
    pub fn get_all_custodians(env: Env) -> Result<Vec<CustodianInfo>, String> {
        let custodian_key = Symbol::new(&env, "custodian");
        let custodians: Map<u32, CustodianInfo> = env.storage()
            .instance()
            .get(&custodian_key)
            .ok_or("Custodians not initialized")?;

        let mut result = Vec::new(&env);
        
        for (_, cust) in custodians.iter() {
            if cust.status == CUSTODIAN_STATUS_ACTIVE {
                result.push_back(cust);
            }
        }

        Ok(result)
    }

    /// Check for expired attestations and create alerts
    pub fn check_expired_attestations(env: Env) -> Result<Vec<CustodyAlert>, String> {
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        let current_time = env.ledger().timestamp();
        let expiry_window: u64 = 86400; // 24 hours in seconds
        let mut alerts = Vec::new(&env);

        for (att_id, att) in attestations.iter() {
            // Check if attestation is older than 24 hours and not verified
            if current_time - att.timestamp > expiry_window 
                && att.status == ATTESTATION_STATUS_PENDING {
                
                let alert = CustodyAlert {
                    asset_id: att.asset_id,
                    alert_type: ALERT_EXPIRED_ATTESTATION,
                    message: String::from("Attestation expired - requires renewal"),
                    timestamp: current_time,
                    severity: SEVERITY_HIGH,
                    resolved: false,
                };
                alerts.push_back(alert);
            }
        }

        Ok(alerts)
    }

    /// Get attestation by ID
    pub fn get_attestation(env: Env, attestation_id: u64) -> Result<CustodyAttestation, String> {
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        attestations.get(attestation_id).ok_or("Attestation not found")
    }

    /// Update asset value (for depreciation tracking)
    pub fn update_asset_value(
        env: Env,
        caller: Address,
        attestation_id: u64,
        new_value: U256,
    ) -> Result<bool, String> {
        let attestation_key = Symbol::new(&env, "attestation");
        let attestations: Map<u64, CustodyAttestation> = env.storage()
            .instance()
            .get(&attestation_key)
            .ok_or("Attestations not initialized")?;

        let mut attestation = attestations.get(attestation_id).ok_or("Attestation not found")?;
        
        // Only the original custodian can update value
        if caller != attestation.custodian {
            return Err("Only original custodian can update value".to_string());
        }

        // Check for significant depreciation (>20%)
        let old_value = attestation.value;
        let depreciation_threshold = old_value / 5; // 20%
        
        if old_value - new_value > depreciation_threshold {
            // Create high severity alert
            let alert = CustodyAlert {
                asset_id: attestation.asset_id.clone(),
                alert_type: ALERT_VALUE_DEVIATION,
                message: String::from("Significant asset depreciation detected"),
                timestamp: env.ledger().timestamp(),
                severity: SEVERITY_CRITICAL,
                resolved: false,
            };
            
            // Store alert
            let alert_key = Symbol::new(&env, "alerts");
            let mut alerts: Vec<CustodyAlert> = env.storage()
                .instance()
                .get(&alert_key)
                .unwrap_or(Vec::new(&env));
            
            alerts.push_back(alert);
            env.storage().instance().set(&alert_key, &alerts);
        }

        attestation.value = new_value;
        
        let mut updated_attestations = attestations;
        updated_attestations.set(attestation_id, attestation);
        env.storage().instance().set(&attestation_key, &updated_attestations);

        Ok(true)
    }

    /// Get all alerts
    pub fn get_alerts(env: Env) -> Result<Vec<CustodyAlert>, String> {
        let alert_key = Symbol::new(&env, "alerts");
        let alerts: Vec<CustodyAlert> = env.storage()
            .instance()
            .get(&alert_key)
            .unwrap_or(Vec::new(&env));
        
        Ok(alerts)
    }

    /// Get platform statistics
    pub fn get_stats(env: Env) -> Result<Map<String, U256>, String> {
        let mut stats = Map::new(&env);
        
        let custodian_count: u32 = env.storage()
            .instance()
            .get(&Symbol::new(&env, "custodian_count"))
            .unwrap_or(0);
        
        let attestation_count: u64 = env.storage()
            .instance()
            .get(&Symbol::new(&env, "attestation_count"))
            .unwrap_or(0);
        
        stats.set(String::from("custodians"), U256::from_u32(custodian_count));
        stats.set(String::from("attestations"), U256::from_u64(attestation_count));
        
        Ok(stats)
    }
}

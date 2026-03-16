use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, String, Vec, Map, U256, u64};

#[contract]
pub struct SupplyChainTracker;

#[derive(Clone)]
pub struct SupplyShipment {
    pub id: String,
    pub donor_id: String,
    pub supply_type: String,
    pub quantity: U256,
    pub unit: String,
    pub origin: Location,
    pub destination: Location,
    pub created_at: u64,
    pub estimated_arrival: u64,
    pub current_status: String, // "in_transit", "at_checkpoint", "delivered", "lost"
    pub checkpoints: Vec<Checkpoint>,
    pub assigned_transporter: Option<Address>,
    pub temperature_requirements: Option<TemperatureRequirements>,
    pub special_handling: Vec<String>,
}

#[derive(Clone)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub facility_name: String,
    pub contact_person: String,
}

#[derive(Clone)]
pub struct Checkpoint {
    pub id: String,
    pub location: Location,
    pub timestamp: u64,
    pub verified_by: Address,
    pub quantity_verified: U256,
    pub condition: String, // "good", "damaged", "partial_loss"
    pub photos: Vec<String>, // IPFS hashes or similar
    pub notes: String,
    pub temperature: Option<f64>,
}

#[derive(Clone)]
pub struct TemperatureRequirements {
    pub min_temp: f64,
    pub max_temp: f64,
    pub critical: bool, // Whether temperature excursions compromise the supply
}

#[derive(Clone)]
pub struct RecipientConfirmation {
    pub shipment_id: String,
    pub recipient_id: String,
    pub received_quantity: U256,
    pub received_at: u64,
    pub condition_report: String,
    pub confirmed_by: Address,
    pub photos: Vec<String>,
}

#[contractimpl]
impl SupplyChainTracker {
    /// Create a new supply shipment
    pub fn create_shipment(
        env: Env,
        donor: Address,
        shipment_id: String,
        donor_id: String,
        supply_type: String,
        quantity: U256,
        unit: String,
        origin: Location,
        destination: Location,
        estimated_arrival: u64,
        temperature_requirements: Option<TemperatureRequirements>,
        special_handling: Vec<String>,
    ) {
        donor.require_auth();
        
        // Check for duplicate shipment
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        if shipments.contains_key(shipment_id.clone()) {
            panic_with_error!(&env, "Shipment already exists");
        }
        
        // Create shipment
        let shipment = SupplyShipment {
            id: shipment_id.clone(),
            donor_id,
            supply_type,
            quantity,
            unit,
            origin,
            destination,
            created_at: env.ledger().timestamp(),
            estimated_arrival,
            current_status: String::from_str(&env, "in_transit"),
            checkpoints: Vec::new(&env),
            assigned_transporter: None,
            temperature_requirements,
            special_handling,
        };
        
        shipments.set(shipment_id.clone(), shipment);
        env.storage().instance().set(&shipments_key, &shipments);
    }

    /// Add checkpoint to shipment journey
    pub fn add_checkpoint(
        env: Env,
        verifier: Address,
        shipment_id: String,
        location: Location,
        quantity_verified: U256,
        condition: String,
        photos: Vec<String>,
        notes: String,
        temperature: Option<f64>,
    ) {
        verifier.require_auth();
        
        let shipments_key = Symbol::new(&env, "shipments");
        let mut shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let mut shipment = match shipments.get(shipment_id.clone()) {
            Some(s) => s,
            None => panic_with_error!(&env, "Shipment not found"),
        };
        
        // Verify temperature requirements if applicable
        if let (Some(temp_req), Some(current_temp)) = (&shipment.temperature_requirements, temperature) {
            if temp_req.critical && (current_temp < temp_req.min_temp || current_temp > temp_req.max_temp) {
                panic_with_error!(&env, "Temperature outside required range");
            }
        }
        
        // Create checkpoint
        let checkpoint_id = format!("cp_{}_{}", shipment_id, env.ledger().timestamp());
        let checkpoint = Checkpoint {
            id: checkpoint_id,
            location,
            timestamp: env.ledger().timestamp(),
            verified_by: verifier,
            quantity_verified,
            condition,
            photos,
            notes,
            temperature,
        };
        
        // Add to checkpoints
        shipment.checkpoints.push_back(checkpoint);
        
        // Update status based on progress
        if shipment.checkpoints.len() > 2 {
            shipment.current_status = String::from_str(&env, "at_checkpoint");
        }
        
        shipments.set(shipment_id, shipment);
        env.storage().instance().set(&shipments_key, &shipments);
    }

    /// Assign transporter to shipment
    pub fn assign_transporter(
        env: Env,
        donor: Address,
        shipment_id: String,
        transporter: Address,
    ) {
        donor.require_auth();
        
        let shipments_key = Symbol::new(&env, "shipments");
        let mut shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut shipment) = shipments.get(shipment_id.clone()) {
            shipment.assigned_transporter = Some(transporter);
            shipments.set(shipment_id, shipment);
            env.storage().instance().set(&shipments_key, &shipments);
        }
    }

    /// Confirm final delivery
    pub fn confirm_delivery(
        env: Env,
        recipient: Address,
        shipment_id: String,
        recipient_id: String,
        received_quantity: U256,
        condition_report: String,
        photos: Vec<String>,
    ) {
        recipient.require_auth();
        
        let shipments_key = Symbol::new(&env, "shipments");
        let mut shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let mut shipment = match shipments.get(shipment_id.clone()) {
            Some(s) => s,
            None => panic_with_error!(&env, "Shipment not found"),
        };
        
        // Create recipient confirmation
        let confirmation = RecipientConfirmation {
            shipment_id: shipment_id.clone(),
            recipient_id,
            received_quantity,
            received_at: env.ledger().timestamp(),
            condition_report,
            confirmed_by: recipient,
            photos,
        };
        
        // Store confirmation
        let confirmations_key = Symbol::new(&env, "confirmations");
        let mut confirmations: Map<String, RecipientConfirmation> = env.storage().instance()
            .get(&confirmations_key)
            .unwrap_or(Map::new(&env));
        
        confirmations.set(shipment_id.clone(), confirmation);
        env.storage().instance().set(&confirmations_key, &confirmations);
        
        // Update shipment status
        shipment.current_status = String::from_str(&env, "delivered");
        shipments.set(shipment_id, shipment);
        env.storage().instance().set(&shipments_key, &shipments);
    }

    /// Get shipment details
    pub fn get_shipment(env: Env, shipment_id: String) -> Option<SupplyShipment> {
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        shipments.get(shipment_id)
    }

    /// Get shipment history with all checkpoints
    pub fn get_shipment_history(env: Env, shipment_id: String) -> (Option<SupplyShipment>, Option<RecipientConfirmation>) {
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let confirmations_key = Symbol::new(&env, "confirmations");
        let confirmations: Map<String, RecipientConfirmation> = env.storage().instance()
            .get(&confirmations_key)
            .unwrap_or(Map::new(&env));
        
        (shipments.get(shipment_id.clone()), confirmations.get(shipment_id))
    }

    /// Track shipments by current location (geographic search)
    pub fn track_by_location(
        env: Env,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Vec<SupplyShipment> {
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let mut nearby_shipments = Vec::new(&env);
        
        for (_, shipment) in shipments.iter() {
            // Check latest checkpoint location
            if let Some(latest_checkpoint) = shipment.checkpoints.get(shipment.checkpoints.len() - 1) {
                let distance = Self::calculate_distance(
                    latitude, longitude,
                    latest_checkpoint.location.latitude, latest_checkpoint.location.longitude
                );
                
                if distance <= radius_km {
                    nearby_shipments.push_back(shipment);
                }
            }
        }
        
        nearby_shipments
    }

    /// Simple distance calculation
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

    /// Get all active shipments
    pub fn get_active_shipments(env: Env) -> Vec<SupplyShipment> {
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let mut active_shipments = Vec::new(&env);
        for (_, shipment) in shipments.iter() {
            if shipment.current_status != String::from_str(&env, "delivered") 
                && shipment.current_status != String::from_str(&env, "lost") {
                active_shipments.push_back(shipment);
            }
        }
        active_shipments
    }

    /// Report shipment as lost
    pub fn report_lost(
        env: Env,
        reporter: Address,
        shipment_id: String,
        reason: String,
    ) {
        reporter.require_auth();
        
        let shipments_key = Symbol::new(&env, "shipments");
        let mut shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut shipment) = shipments.get(shipment_id.clone()) {
            shipment.current_status = String::from_str(&env, "lost");
            
            // Add loss as final checkpoint
            let loss_checkpoint = Checkpoint {
                id: format!("loss_{}", shipment_id),
                location: shipment.destination, // Assume lost near destination
                timestamp: env.ledger().timestamp(),
                verified_by: reporter,
                quantity_verified: U256::from_u64(0),
                condition: String::from_str(&env, "lost"),
                photos: Vec::new(&env),
                notes: reason,
                temperature: None,
            };
            
            shipment.checkpoints.push_back(loss_checkpoint);
            shipments.set(shipment_id, shipment);
            env.storage().instance().set(&shipments_key, &shipments);
        }
    }

    /// Get shipments by donor
    pub fn get_shipments_by_donor(env: Env, donor_id: String) -> Vec<SupplyShipment> {
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let mut donor_shipments = Vec::new(&env);
        for (_, shipment) in shipments.iter() {
            if shipment.donor_id == donor_id {
                donor_shipments.push_back(shipment);
            }
        }
        donor_shipments
    }

    /// Get temperature alerts for cold chain shipments
    pub fn get_temperature_alerts(env: Env) -> Vec<(String, String)> {
        let shipments_key = Symbol::new(&env, "shipments");
        let shipments: Map<String, SupplyShipment> = env.storage().instance()
            .get(&shipments_key)
            .unwrap_or(Map::new(&env));
        
        let mut alerts = Vec::new(&env);
        
        for (shipment_id, shipment) in shipments.iter() {
            if let Some(temp_req) = &shipment.temperature_requirements {
                if let Some(latest_checkpoint) = shipment.checkpoints.get(shipment.checkpoints.len() - 1) {
                    if let Some(current_temp) = latest_checkpoint.temperature {
                        if current_temp < temp_req.min_temp || current_temp > temp_req.max_temp {
                            let alert_msg = format!("Temperature breach: {}°C (required: {}-{}°C)", 
                                current_temp, temp_req.min_temp, temp_req.max_temp);
                            alerts.push_back((shipment_id.clone(), String::from_str(&env, &alert_msg)));
                        }
                    }
                }
            }
        }
        
        alerts
    }
}

import { 
  Server, 
  TransactionBuilder, 
  Networks, 
  Keypair, 
  Contract,
  Address,
  nativeToScVal,
  scValToNative
} from 'stellar-sdk';

// Types for custody operations
export interface CustodianProfile {
  name: string;
  jurisdiction: string;
  license: string;
  credentials: string[];
}

export interface CustodianInfo {
  name: string;
  jurisdiction: string;
  license: string;
  reputation_score: number;
  status: number;
  registered_date: number;
  total_attestations: number;
  successful_verifications: number;
  failed_verifications: number;
}

export interface CustodyAttestation {
  asset_id: string;
  custodian: string;
  location: string;
  condition: string;
  value: string;
  timestamp: number;
  proof_hash: string;
  asset_type: number;
  status: number;
  verification_count: number;
  signatures: string[];
}

export interface ProofData {
  documentHash: string;
  merkleProof?: string[];
  zkProof?: string;
  timestamp: number;
  notarySignature?: string;
  iotData?: {
    temperature?: number;
    location?: string;
    motion?: boolean;
  };
  satelliteData?: string;
}

export interface AttestationRequest {
  assetId: string;
  location: string;
  condition: string;
  value: string;
  proofData: ProofData;
  assetType: number;
}

export interface DisputeRequest {
  assetId: string;
  reason: string;
  bond: string;
}

export interface CustodyAlert {
  asset_id: string;
  alert_type: number;
  message: string;
  timestamp: number;
  severity: number;
  resolved: boolean;
}

// Asset types
export const ASSET_TYPE_REAL_ESTATE = 0;
export const ASSET_TYPE_PRECIOUS_METALS = 1;
export const ASSET_TYPE_ART = 2;
export const ASSET_TYPE_COMMODITIES = 3;
export const ASSET_TYPE_INVOICE = 4;

// Custodian status
export const CUSTODIAN_STATUS_PENDING = 0;
export const CUSTODIAN_STATUS_ACTIVE = 1;
export const CUSTODIAN_STATUS_SUSPENDED = 2;
export const CUSTODIAN_STATUS_REVOKED = 3;

// Attestation status
export const ATTESTATION_STATUS_PENDING = 0;
export const ATTESTATION_STATUS_VERIFIED = 1;
export const ATTESTATION_STATUS_DISPUTED = 2;
export const ATTESTATION_STATUS_EXPIRED = 3;
export const ATTESTATION_STATUS_SLASHED = 4;

// Alert types
export const ALERT_EXPIRED_ATTESTATION = 0;
export const ALERT_MISSING_ATTESTATION = 1;
export const ALERT_CUSTODIAN_SUSPENDED = 2;
export const ALERT_VALUE_DEVIATION = 3;
export const ALERT_INSURANCE_EXPIRED = 4;

// Severity levels
export const SEVERITY_LOW = 0;
export const SEVERITY_MEDIUM = 1;
export const SEVERITY_HIGH = 2;
export const SEVERITY_CRITICAL = 3;

export class CustodyClient {
  private server: Server;
  private contract: Contract;
  private config: any;

  constructor(config: any) {
    this.config = config;
    this.server = new Server(config.rpcUrl);
    this.contract = new Contract(config.contractIds.custodyValidator);
  }

  private getNetworkPassphrase(): string {
    return this.config.network === 'mainnet' 
      ? Networks.PUBLIC 
      : Networks.TESTNET;
  }

  /**
   * Register a new custodian (admin only)
   */
  async registerCustodian(
    adminKey: string,
    profile: CustodianProfile
  ): Promise<number> {
    const adminKeypair = Keypair.fromSecret(adminKey);
    const adminAccount = await this.server.getAccount(adminKeypair.publicKey());

    const tx = new TransactionBuilder(adminAccount, {
      fee: '100',
      networkPassphrase: this.getNetworkPassphrase(),
    })
      .addOperation(
        this.contract.call(
          "register_custodian",
          new Address(adminKeypair.publicKey()).toScVal(),
          nativeToScVal(profile.name),
          nativeToScVal(profile.jurisdiction),
          nativeToScVal(profile.license)
        )
      )
      .setTimeout(30)
      .build();

    tx.sign(adminKeypair);
    const result = await this.server.sendTransaction(tx);
    
    if (result.status === 'SUCCESS') {
      // Parse the return value to get custodian ID
      const parsed = scValToNative(result.returnValue);
      return parsed;
    } else {
      throw new Error(`Failed to register custodian: ${result.status}`);
    }
  }

  /**
   * Submit an attestation for an asset
   */
  async submitAttestation(
    custodianKey: string,
    custodianId: number,
    request: AttestationRequest
  ): Promise<number> {
    const custodianKeypair = Keypair.fromSecret(custodianKey);
    const custodianAccount = await this.server.getAccount(custodianKeypair.publicKey());

    // Generate proof hash from proof data
    const proofHash = this.generateProofHash(request.proofData);

    const tx = new TransactionBuilder(custodianAccount, {
      fee: '100',
      networkPassphrase: this.getNetworkPassphrase(),
    })
      .addOperation(
        this.contract.call(
          "submit_attestation",
          new Address(custodianKeypair.publicKey()).toScVal(),
          nativeToScVal(custodianId),
          nativeToScVal(request.assetId),
          nativeToScVal(request.location),
          nativeToScVal(request.condition),
          nativeToScVal(request.value),
          nativeToScVal(proofHash),
          nativeToScVal(request.assetType)
        )
      )
      .setTimeout(30)
      .build();

    tx.sign(custodianKeypair);
    const result = await this.server.sendTransaction(tx);
    
    if (result.status === 'SUCCESS') {
      const parsed = scValToNative(result.returnValue);
      return parsed;
    } else {
      throw new Error(`Failed to submit attestation: ${result.status}`);
    }
  }

  /**
   * Verify an attestation (multi-sig, 3-of-5)
   */
  async verifyAttestation(
    verifierKey: string,
    attestationId: number
  ): Promise<boolean> {
    const verifierKeypair = Keypair.fromSecret(verifierKey);
    const verifierAccount = await this.server.getAccount(verifierKeypair.publicKey());

    const tx = new TransactionBuilder(verifierAccount, {
      fee: '100',
      networkPassphrase: this.getNetworkPassphrase(),
    })
      .addOperation(
        this.contract.call(
          "verify_attestation",
          new Address(verifierKeypair.publicKey()).toScVal(),
          nativeToScVal(attestationId)
        )
      )
      .setTimeout(30)
      .build();

    tx.sign(verifierKeypair);
    const result = await this.server.sendTransaction(tx);
    
    if (result.status === 'SUCCESS') {
      const parsed = scValToNative(result.returnValue);
      return parsed;
    } else {
      throw new Error(`Failed to verify attestation: ${result.status}`);
    }
  }

  /**
   * Verify asset backing - check all attestations current and valid
   */
  async verifyAssetBacking(assetId: string): Promise<boolean> {
    const result = await this.server.simulateTransaction(
      this.contract.call("verify_asset_backing", nativeToScVal(assetId))
    );
    
    if (result.result) {
      return scValToNative(result.result);
    }
    return false;
  }

  /**
   * Get custody history for an asset
   */
  async getCustodyHistory(assetId: string): Promise<CustodyAttestation[]> {
    const result = await this.server.simulateTransaction(
      this.contract.call("get_custody_history", nativeToScVal(assetId))
    );
    
    if (result.result) {
      return scValToNative(result.result);
    }
    return [];
  }

  /**
   * Initiate a dispute with bond staking
   */
  async initiateDispute(
    challengerKey: string,
    request: DisputeRequest
  ): Promise<string> {
    const challengerKeypair = Keypair.fromSecret(challengerKey);
    const challengerAccount = await this.server.getAccount(challengerKeypair.publicKey());

    const tx = new TransactionBuilder(challengerAccount, {
      fee: '100',
      networkPassphrase: this.getNetworkPassphrase(),
    })
      .addOperation(
        this.contract.call(
          "dispute_attestation",
          new Address(challengerKeypair.publicKey()).toScVal(),
          nativeToScVal(request.assetId),
          nativeToScVal(request.reason),
          nativeToScVal(request.bond)
        )
      )
      .setTimeout(30)
      .build();

    tx.sign(challengerKeypair);
    const result = await this.server.sendTransaction(tx);
    
    if (result.status === 'SUCCESS') {
      const parsed = scValToNative(result.returnValue);
      return parsed;
    } else {
      throw new Error(`Failed to initiate dispute: ${result.status}`);
    }
  }

  /**
   * Get custodian info
   */
  async getCustodian(custodianId: number): Promise<CustodianInfo> {
    const result = await this.server.simulateTransaction(
      this.contract.call("get_custodian", nativeToScVal(custodianId))
    );
    
    if (result.result) {
      return scValToNative(result.result);
    }
    throw new Error("Failed to get custodian");
  }

  /**
   * Get all active custodians
   */
  async getAllCustodians(): Promise<CustodianInfo[]> {
    const result = await this.server.simulateTransaction(
      this.contract.call("get_all_custodians")
    );
    
    if (result.result) {
      return scValToNative(result.result);
    }
    return [];
  }

  /**
   * Get attestation by ID
   */
  async getAttestation(attestationId: number): Promise<CustodyAttestation> {
    const result = await this.server.simulateTransaction(
      this.contract.call("get_attestation", nativeToScVal(attestationId))
    );
    
    if (result.result) {
      return scValToNative(result.result);
    }
    throw new Error("Failed to get attestation");
  }

  /**
   * Get platform statistics
   */
  async getStats(): Promise<Record<string, string>> {
    const result = await this.server.simulateTransaction(
      this.contract.call("get_stats")
    );
    
    if (result.result) {
      return scValToNative(result.result);
    }
    return {};
  }

  /**
   * Update asset value (for depreciation tracking)
   */
  async updateAssetValue(
    custodianKey: string,
    attestationId: number,
    newValue: string
  ): Promise<boolean> {
    const custodianKeypair = Keypair.fromSecret(custodianKey);
    const custodianAccount = await this.server.getAccount(custodianKeypair.publicKey());

    const tx = new TransactionBuilder(custodianAccount, {
      fee: '100',
      networkPassphrase: this.getNetworkPassphrase(),
    })
      .addOperation(
        this.contract.call(
          "update_asset_value",
          new Address(custodianKeypair.publicKey()).toScVal(),
          nativeToScVal(attestationId),
          nativeToScVal(newValue)
        )
      )
      .setTimeout(30)
      .build();

    tx.sign(custodianKeypair);
    const result = await this.server.sendTransaction(tx);
    
    if (result.status === 'SUCCESS') {
      const parsed = scValToNative(result.returnValue);
      return parsed;
    } else {
      throw new Error(`Failed to update asset value: ${result.status}`);
    }
  }

  /**
   * Generate proof hash from proof data
   */
  private generateProofHash(proofData: ProofData): string {
    // In production, this would use proper cryptographic hashing
    // For now, create a composite hash
    const data = JSON.stringify({
      documentHash: proofData.documentHash,
      timestamp: proofData.timestamp,
      iotData: proofData.iotData
    });
    
    // Simple hash for demonstration - use proper crypto in production
    let hash = 0;
    for (let i = 0; i < data.length; i++) {
      const char = data.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return Math.abs(hash).toString(16).padStart(16, '0');
  }

  /**
   * Verify cryptographic proof
   */
  async verifyProof(proofData: ProofData): Promise<boolean> {
    // In production, this would verify:
    // - Merkle proofs for document authenticity
    // - Zero-knowledge proofs for sensitive data
    // - Photo/video hash anchoring with timestamp
    // - Notary public signature verification
    
    // For now, basic verification
    if (!proofData.documentHash || !proofData.timestamp) {
      return false;
    }
    
    // Check timestamp is not too old (24 hours)
    const now = Date.now();
    const proofTime = proofData.timestamp * 1000;
    const age = now - proofTime;
    
    return age < 24 * 60 * 60 * 1000; // 24 hours
  }
}

/**
 * Custody Network SDK - Higher level interface
 */
export class CustodyNetworkSDK {
  private custodyClient: CustodyClient;
  private config: any;

  constructor(config: any) {
    this.config = config;
    this.custodyClient = new CustodyClient(config);
  }

  /**
   * Full attestation workflow with multi-sig
   */
  async submitAndVerify(
    custodianKeys: string[],
    custodianId: number,
    request: AttestationRequest
  ): Promise<{ attestationId: number; verified: boolean }> {
    // Submit attestation
    const attestationId = await this.custodyClient.submitAttestation(
      custodianKeys[0],
      custodianId,
      request
    );

    // Get verifiers (different from submitter)
    const verifiers = custodianKeys.slice(1, 4); // 3 verifiers
    
    let verified = false;
    for (const verifierKey of verifiers) {
      try {
        verified = await this.custodyClient.verifyAttestation(
          verifierKey,
          attestationId
        );
        if (verified) break;
      } catch (e) {
        console.log(`Verification failed for ${verifierKey}:`, e);
      }
    }

    return { attestationId, verified };
  }

  /**
   * Check if asset is fully backed
   */
  async isAssetFullyBacked(assetId: string): Promise<{
    backed: boolean;
    history: CustodyAttestation[];
  }> {
    const history = await this.custodyClient.getCustodyHistory(assetId);
    const backed = await this.custodyClient.verifyAssetBacking(assetId);
    
    return { backed, history };
  }

  /**
   * Get asset health score
   */
  async getAssetHealthScore(assetId: string): Promise<number> {
    const history = await this.custodyClient.getCustodyHistory(assetId);
    
    if (history.length === 0) return 0;
    
    // Calculate health based on:
    // - Verification count
    // - Time since last attestation
    // - Dispute history
    
    const latest = history[history.length - 1];
    const now = Date.now();
    const age = now - (latest.timestamp * 1000);
    const ageScore = Math.max(0, 100 - (age / (24 * 60 * 60 * 1000))); // Decay over days
    
    const verificationScore = Math.min(100, latest.verification_count * 20);
    const statusScore = latest.status === ATTESTATION_STATUS_VERIFIED ? 100 : 0;
    
    return Math.round((ageScore + verificationScore + statusScore) / 3);
  }
}

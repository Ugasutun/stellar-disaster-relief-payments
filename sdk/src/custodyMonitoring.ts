import { CustodyClient, CustodyAlert, CustodianInfo, CustodyAttestation } from './custody';

// Monitoring configuration
export interface MonitoringConfig {
  checkIntervalMs: number;
  alertThresholds: {
    expiredAttestationHours: number;
    minReputationScore: number;
    maxValueDeviationPercent: number;
  };
  notificationCallbacks?: {
    onAlert?: (alert: CustodyAlert) => void;
    onCustodianUpdate?: (custodian: CustodianInfo) => void;
    onDepreciation?: (assetId: string, oldValue: string, newValue: string) => void;
  };
}

export interface AssetMonitoringState {
  assetId: string;
  lastAttestationTime: number;
  lastValue: string;
  alertCount: number;
  healthScore: number;
}

export interface CustodianMonitoringState {
  custodianId: number;
  reputationScore: number;
  activeAttestations: number;
  lastActivityTime: number;
}

export class CustodyMonitoring {
  private custodyClient: CustodyClient;
  private config: MonitoringConfig;
  private assetStates: Map<string, AssetMonitoringState> = new Map();
  private custodianStates: Map<number, CustodianMonitoringState> = new Map();
  private monitoringInterval: NodeJS.Timeout | null = null;
  private isRunning: boolean = false;

  constructor(custodyClient: CustodyClient, config: Partial<MonitoringConfig> = {}) {
    this.custodyClient = custodyClient;
    this.config = {
      checkIntervalMs: config.checkIntervalMs || 3600000, // 1 hour default
      alertThresholds: {
        expiredAttestationHours: config.alertThresholds?.expiredAttestationHours || 24,
        minReputationScore: config.alertThresholds?.minReputationScore || 70,
        maxValueDeviationPercent: config.alertThresholds?.maxValueDeviationPercent || 20,
      },
      notificationCallbacks: config.notificationCallbacks
    };
  }

  /**
   * Start automated monitoring
   */
  start(): void {
    if (this.isRunning) return;
    
    this.isRunning = true;
    this.monitoringInterval = setInterval(
      () => this.runMonitoringCycle(),
      this.config.checkIntervalMs
    );
    
    // Run immediately
    this.runMonitoringCycle();
  }

  /**
   * Stop monitoring
   */
  stop(): void {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = null;
    }
    this.isRunning = false;
  }

  /**
   * Run one monitoring cycle
   */
  private async runMonitoringCycle(): Promise<void> {
    try {
      await Promise.all([
        this.checkExpiredAttestations(),
        this.checkCustodianReputation(),
        this.checkAssetValues(),
        this.updateAssetHealth()
      ]);
    } catch (error) {
      console.error('Monitoring cycle failed:', error);
    }
  }

  /**
   * Check for expired attestations
   */
  async checkExpiredAttestations(): Promise<CustodyAlert[]> {
    const alerts = await this.custodyClient.getStats();
    // In production, this would check actual expired attestations
    // For now, return empty array
    return [];
  }

  /**
   * Check custodian reputation scores
   */
  async checkCustodianReputation(): Promise<CustodianInfo[]> {
    const custodians = await this.custodyClient.getAllCustodians();
    const lowReputationCustodians: CustodianInfo[] = [];

    for (const custodian of custodians) {
      // Update state
      const state = this.custodianStates.get(custodian.reputation_score) || {
        custodianId: 0,
        reputationScore: custodian.reputation_score,
        activeAttestations: custodian.total_attestations,
        lastActivityTime: custodian.registered_date
      };
      
      this.custodianStates.set(custodian.reputation_score, state);

      // Check reputation threshold
      if (custodian.reputation_score < this.config.alertThresholds.minReputationScore) {
        lowReputationCustodians.push(custodian);
        
        // Notify
        if (this.config.notificationCallbacks?.onCustodianUpdate) {
          this.config.notificationCallbacks.onCustodianUpdate(custodian);
        }
      }
    }

    return lowReputationCustodians;
  }

  /**
   * Check for asset value changes (depreciation)
   */
  async checkAssetValues(): Promise<void> {
    // This would track asset values over time
    // and alert on significant changes
  }

  /**
   * Update asset health scores
   */
  async updateAssetHealth(): Promise<void> {
    for (const [assetId, state] of this.assetStates) {
      try {
        const history = await this.custodyClient.getCustodyHistory(assetId);
        
        if (history.length > 0) {
          const latest = history[history.length - 1];
          const now = Date.now();
          const age = now - (latest.timestamp * 1000);
          
          // Calculate health score
          const ageScore = Math.max(0, 100 - (age / (24 * 60 * 60 * 1000) * 10));
          const verificationScore = Math.min(100, latest.verification_count * 25);
          const statusScore = latest.status === 1 ? 100 : 0;
          
          state.healthScore = Math.round((ageScore + verificationScore + statusScore) / 3);
          state.lastAttestationTime = latest.timestamp;
          
          this.assetStates.set(assetId, state);
        }
      } catch (error) {
        console.error(`Failed to update health for ${assetId}:`, error);
      }
    }
  }

  /**
   * Track a specific asset
   */
  trackAsset(assetId: string): void {
    this.assetStates.set(assetId, {
      assetId,
      lastAttestationTime: 0,
      lastValue: '0',
      alertCount: 0,
      healthScore: 0
    });
  }

  /**
   * Stop tracking an asset
   */
  untrackAsset(assetId: string): void {
    this.assetStates.delete(assetId);
  }

  /**
   * Get asset monitoring state
   */
  getAssetState(assetId: string): AssetMonitoringState | undefined {
    return this.assetStates.get(assetId);
  }

  /**
   * Get all tracked assets
   */
  getAllTrackedAssets(): AssetMonitoringState[] {
    return Array.from(this.assetStates.values());
  }

  /**
   * Get custodian monitoring state
   */
  getCustodianState(custodianId: number): CustodianMonitoringState | undefined {
    return this.custodianStates.get(custodianId);
  }

  /**
   * Check insurance status (placeholder for insurance integration)
   */
  async checkInsuranceStatus(assetId: string): Promise<{
    insured: boolean;
    provider?: string;
    expiryDate?: number;
    claimTriggered: boolean;
  }> {
    // In production, this would integrate with insurance providers
    // via oracles or API calls
    
    return {
      insured: false,
      claimTriggered: false
    };
  }

  /**
   * Trigger insurance claim on custody failure
   */
  async triggerInsuranceClaim(
    assetId: string,
    failureReason: string
  ): Promise<{ claimId: string; status: string }> {
    // In production, this would:
    // 1. Verify custody failure occurred
    // 2. Submit claim to insurance provider
    // 3. Return claim ID for tracking
    
    const claimId = `CLAIM-${assetId}-${Date.now()}`;
    
    return {
      claimId,
      status: 'submitted'
    };
  }

  /**
   * Generate monitoring report
   */
  async generateReport(): Promise<{
    totalAssets: number;
    healthyAssets: number;
    atRiskAssets: number;
    custodianCount: number;
    lowReputationCustodians: number;
    alerts: CustodyAlert[];
  }> {
    const custodians = await this.custodyClient.getAllCustodians();
    const alerts = await this.custodyClient.getStats().then(() => []);
    
    let healthyAssets = 0;
    let atRiskAssets = 0;
    
    for (const state of this.assetStates.values()) {
      if (state.healthScore >= 70) {
        healthyAssets++;
      } else {
        atRiskAssets++;
      }
    }
    
    const lowReputationCustodians = custodians.filter(
      c => c.reputation_score < this.config.alertThresholds.minReputationScore
    ).length;
    
    return {
      totalAssets: this.assetStates.size,
      healthyAssets,
      atRiskAssets,
      custodianCount: custodians.length,
      lowReputationCustodians,
      alerts
    };
  }
}

/**
 * Insurance integration service
 */
export class InsuranceIntegration {
  private custodyClient: CustodyClient;
  private monitoring: CustodyMonitoring;

  constructor(custodyClient: CustodyClient, monitoring: CustodyMonitoring) {
    this.custodyClient = custodyClient;
    this.monitoring = monitoring;
  }

  /**
   * Verify insurance coverage for asset
   */
  async verifyCoverage(assetId: string): Promise<{
    covered: boolean;
    policyDetails?: {
      provider: string;
      policyNumber: string;
      coverageAmount: string;
      expiryDate: number;
    };
  }> {
    // In production, verify with insurance oracle/API
    return { covered: false };
  }

  /**
   * Monitor and auto-trigger claims on custody failure
   */
  async setupAutoClaim(
    assetId: string,
    failureConditions: {
      expiredAttestationHours: number;
      minHealthScore: number;
    }
  ): Promise<void> {
    // Set up monitoring to detect failure conditions
    this.monitoring.trackAsset(assetId);
    
    // In production, this would set up automated checks
    // and trigger claims when conditions are met
  }

  /**
   * Get claim status
   */
  async getClaimStatus(claimId: string): Promise<{
    status: 'pending' | 'approved' | 'rejected' | 'paid';
    amount?: string;
    timeline?: string;
  }> {
    // In production, query insurance provider
    return { status: 'pending' };
  }
}

/**
 * Create custody monitoring instance
 */
export const createCustodyMonitoring = (
  custodyClient: CustodyClient,
  config?: Partial<MonitoringConfig>
): CustodyMonitoring => {
  return new CustodyMonitoring(custodyClient, config);
};

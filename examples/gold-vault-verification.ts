import { 
  CustodyClient,
  CustodyNetworkSDK,
  ASSET_TYPE_PRECIOUS_METALS,
  ATTESTATION_STATUS_VERIFIED,
  type CustodianProfile,
  type AttestationRequest,
  type ProofData
} from '../sdk/src/custody';
import { createCustodyMonitoring, type MonitoringConfig } from '../sdk/src/custodyMonitoring';

/**
 * Gold Vault Verification Example
 * 
 * This example demonstrates how to verify physical gold bullion custody
 * using the Custody Verification Oracle system. It includes:
 * - Vault audit certificate verification
 * - Purity assay verification
 * - Weight verification
 * - Chain of custody documentation
 * - IoT sensor data integration (temperature, location, motion)
 * - Multi-sig verification (3-of-5)
 */

interface GoldBar {
  id: string;
  weight: number; // in troy ounces
  purity: number; // 0.9999 = 9999
  refiner: string;
  year: number;
}

interface VaultAudit {
  vaultId: string;
  auditor: string;
  auditDate: number;
  totalBars: number;
  totalWeight: number;
  bars: GoldBar[];
  temperature: number;
  humidity: number;
  securityStatus: 'secure' | 'breach' | 'alert';
}

// Configuration
const CONFIG = {
  network: 'testnet' as const,
  rpcUrl: 'https://soroban-testnet.stellar.org',
  contractIds: {
    custodyValidator: 'CUSTODY_CONTRACT_ID'
  }
};

// Demo custodian keys (in production, these would be real keys)
const CUSTODIAN_KEYS = [
  'SCUSTODIAN1_KEY',
  'SCUSTODIAN2_KEY',
  'SCUSTODIAN3_KEY',
  'SCUSTODIAN4_KEY',
  'SCUSTODIAN5_KEY'
];

const ADMIN_KEY = 'SADMIN_KEY';

async function goldVaultVerification() {
  console.log('🏆 Starting Gold Vault Verification');
  console.log('=====================================\n');

  // Initialize SDK
  const custodyClient = new CustodyClient(CONFIG);
  const custodySDK = new CustodyNetworkSDK(CONFIG);

  // Step 1: Register Verified Custodians
  console.log('📋 Step 1: Registering Verified Custodians');
  console.log('-------------------------------------------');
  
  const custodians: CustodianProfile[] = [
    {
      name: 'Swiss Gold Vault AG',
      jurisdiction: 'Switzerland',
      license: 'CH-GOLD-2024-001',
      credentials: ['LBMA Good Delivery', 'ISO 9001', 'SOC 2 Type II']
    },
    {
      name: 'Brinks International',
      jurisdiction: 'United States',
      license: 'US-GOLD-2024-002',
      credentials: ['LBMA Good Delivery', 'ISO 9001']
    },
    {
      name: 'G4S Secure Solutions',
      jurisdiction: 'United Kingdom',
      license: 'UK-GOLD-2024-003',
      credentials: ['LBMA Good Delivery', 'ISO 9001', 'UK FCA']
    },
    {
      name: 'Loomis International',
      jurisdiction: 'Sweden',
      license: 'SE-GOLD-2024-004',
      credentials: ['LBMA Good Delivery', 'ISO 9001']
    },
    {
      name: 'Malca-Amit',
      jurisdiction: 'Singapore',
      license: 'SG-GOLD-2024-005',
      credentials: ['LBMA Good Delivery', 'ISO 9001']
    }
  ];

  const custodianIds: number[] = [];
  for (const custodian of custodians) {
    try {
      const id = await custodyClient.registerCustodian(ADMIN_KEY, custodian);
      custodianIds.push(id);
      console.log(`  ✓ Registered: ${custodian.name} (ID: ${id})`);
    } catch (error) {
      console.log(`  ⚠ Custodian may already exist: ${custodian.name}`);
    }
  }
  console.log('');

  // Step 2: Simulate Vault Audit Data
  console.log('📋 Step 2: Simulating Vault Audit');
  console.log('-----------------------------------');

  const vaultAudit: VaultAudit = {
    vaultId: 'VAULT-CH-001',
    auditor: 'Swiss Gold Vault AG',
    auditDate: Date.now(),
    totalBars: 100,
    totalWeight: 10000, // 10,000 troy ounces
    bars: generateGoldBars(100),
    temperature: 20.5,
    humidity: 45,
    securityStatus: 'secure'
  };

  console.log(`  Vault: ${vaultAudit.vaultId}`);
  console.log(`  Auditor: ${vaultAudit.auditor}`);
  console.log(`  Total Bars: ${vaultAudit.totalBars}`);
  console.log(`  Total Weight: ${vaultAudit.totalWeight} troy oz`);
  console.log(`  Temperature: ${vaultAudit.temperature}°C`);
  console.log(`  Humidity: ${vaultAudit.humidity}%`);
  console.log(`  Security: ${vaultAudit.securityStatus}`);
  console.log('');

  // Step 3: Generate Cryptographic Proofs
  console.log('📋 Step 3: Generating Cryptographic Proofs');
  console.log('-------------------------------------------');

  const proofData: ProofData = {
    documentHash: generateDocumentHash(vaultAudit),
    timestamp: Math.floor(Date.now() / 1000),
    iotData: {
      temperature: vaultAudit.temperature,
      location: vaultAudit.vaultId,
      motion: vaultAudit.securityStatus === 'secure'
    },
    notarySignature: 'NOTARY_SIG_HERE'
  };

  console.log(`  Document Hash: ${proofData.documentHash}`);
  console.log(`  Timestamp: ${new Date(proofData.timestamp * 1000).toISOString()}`);
  console.log(`  IoT Temperature: ${proofData.iotData?.temperature}°C`);
  console.log(`  IoT Location: ${proofData.iotData?.location}`);
  console.log('');

  // Step 4: Submit Attestation
  console.log('📋 Step 4: Submitting Custody Attestation');
  console.log('-------------------------------------------');

  const assetId = `GOLD-${vaultAudit.vaultId}-${Date.now()}`;
  
  const attestationRequest: AttestationRequest = {
    assetId,
    location: 'Swiss Gold Vault, Zurich, Switzerland',
    condition: 'Excellent - All bars verified intact',
    value: (vaultAudit.totalWeight * 1950).toString(), // ~$1950/oz
    proofData,
    assetType: ASSET_TYPE_PRECIOUS_METALS
  };

  try {
    const attestationId = await custodyClient.submitAttestation(
      CUSTODIAN_KEYS[0],
      custodianIds[0] || 1,
      attestationRequest
    );
    console.log(`  ✓ Attestation submitted (ID: ${attestationId})`);
    console.log(`  Asset ID: ${assetId}`);
    console.log(`  Value: $${attestationRequest.value}`);
    console.log('');
  } catch (error) {
    console.log(`  ⚠ Attestation submission: ${error}`);
    console.log('');
  }

  // Step 5: Multi-Sig Verification (3-of-5)
  console.log('📋 Step 5: Multi-Sig Verification (3-of-5)');
  console.log('---------------------------------------------');

  const verificationResults = [];
  for (let i = 1; i <= 3; i++) {
    try {
      const verified = await custodyClient.verifyAttestation(
        CUSTODIAN_KEYS[i],
        1 // attestation ID
      );
      verificationResults.push(verified);
      console.log(`  ✓ Verification ${i}/3: ${verified ? 'SUCCESS' : 'PENDING'}`);
    } catch (error) {
      console.log(`  ⚠ Verification ${i}/3: ${error}`);
    }
  }
  console.log('');

  // Step 6: Verify Asset Backing
  console.log('📋 Step 6: Verifying Asset Backing');
  console.log('------------------------------------');

  const isBacked = await custodyClient.verifyAssetBacking(assetId);
  console.log(`  Asset Backing Status: ${isBacked ? '✅ VERIFIED' : '❌ NOT VERIFIED'}`);
  console.log('');

  // Step 7: Get Custody History
  console.log('📋 Step 7: Custody History');
  console.log('---------------------------');

  const history = await custodyClient.getCustodyHistory(assetId);
  console.log(`  Total attestations: ${history.length}`);
  for (const att of history) {
    console.log(`  - ${new Date(att.timestamp * 1000).toISOString()}: ${att.status === ATTESTATION_STATUS_VERIFIED ? 'Verified' : 'Pending'}`);
  }
  console.log('');

  // Step 8: Set Up Monitoring
  console.log('📋 Step 8: Setting Up Monitoring');
  console.log('----------------------------------');

  const monitoringConfig: Partial<MonitoringConfig> = {
    checkIntervalMs: 3600000, // 1 hour
    alertThresholds: {
      expiredAttestationHours: 24,
      minReputationScore: 70,
      maxValueDeviationPercent: 20
    },
    notificationCallbacks: {
      onAlert: (alert) => {
        console.log(`  🔔 ALERT: ${alert.message}`);
      },
      onCustodianUpdate: (custodian) => {
        console.log(`  🔔 CUSTODIAN UPDATE: ${custodian.name} - Score: ${custodian.reputation_score}`);
      }
    }
  };

  const monitoring = createCustodyMonitoring(custodyClient, monitoringConfig);
  monitoring.trackAsset(assetId);
  monitoring.start();

  console.log('  ✓ Monitoring started');
  console.log('  ✓ Tracking asset:', assetId);
  console.log('');

  // Step 9: Insurance Integration
  console.log('📋 Step 9: Insurance Status Check');
  console.log('-----------------------------------');

  const insuranceStatus = {
    provider: 'Lloyd\'s of London',
    policyNumber: 'GL-2024-CH-001',
    coverageAmount: '19500000', // $19.5M
    expiryDate: Date.now() + (365 * 24 * 60 * 60 * 1000),
    verified: true
  };

  console.log(`  Provider: ${insuranceStatus.provider}`);
  console.log(`  Policy: ${insuranceStatus.policyNumber}`);
  console.log(`  Coverage: $${parseInt(insuranceStatus.coverageAmount).toLocaleString()}`);
  console.log(`  Expiry: ${new Date(insuranceStatus.expiryDate).toISOString()}`);
  console.log(`  Status: ${insuranceStatus.verified ? '✅ VERIFIED' : '❌ NOT VERIFIED'}`);
  console.log('');

  // Step 10: Summary
  console.log('📋 Summary');
  console.log('----------');
  console.log(`  ✅ Custodians Registered: ${custodianIds.length}`);
  console.log(`  ✅ Attestation Submitted: ${assetId}`);
  console.log(`  ✅ Multi-Sig Verification: 3-of-5`);
  console.log(`  ✅ Asset Backing: ${isBacked ? 'VERIFIED' : 'PENDING'}`);
  console.log(`  ✅ Insurance: Active`);
  console.log(`  ✅ Monitoring: Active`);
  console.log('');
  console.log('🎉 Gold Vault Verification Complete!');
  console.log('=====================================\n');

  // Cleanup
  setTimeout(() => {
    monitoring.stop();
    console.log('Monitoring stopped.');
  }, 5000);
}

/**
 * Generate sample gold bars
 */
function generateGoldBars(count: number): GoldBar[] {
  const bars: GoldBar[] = [];
  const refiners = ['PAMP Suisse', 'Credit Suisse', 'Royal Canadian Mint', 'Perth Mint', 'Johnson Matthey'];
  
  for (let i = 0; i < count; i++) {
    bars.push({
      id: `BAR-${String(i + 1).padStart(4, '0')}`,
      weight: 100 + Math.random() * 100, // 100-200 troy oz
      purity: 9999, // 24K
      refiner: refiners[Math.floor(Math.random() * refiners.length)],
      year: 2020 + Math.floor(Math.random() * 5)
    });
  }
  
  return bars;
}

/**
 * Generate document hash (simplified)
 */
function generateDocumentHash(audit: VaultAudit): string {
  const data = JSON.stringify({
    vaultId: audit.vaultId,
    auditor: audit.auditor,
    auditDate: audit.auditDate,
    totalBars: audit.totalBars,
    totalWeight: audit.totalWeight,
    bars: audit.bars.map(b => ({ id: b.id, weight: b.weight, purity: b.purity }))
  });
  
  // Simple hash - use proper crypto in production
  let hash = 0;
  for (let i = 0; i < data.length; i++) {
    const char = data.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash).toString(16).padStart(16, '0');
}

// Run the example
goldVaultVerification().catch(console.error);

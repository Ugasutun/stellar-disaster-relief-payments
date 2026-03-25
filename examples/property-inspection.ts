import { 
  CustodyClient,
  CustodyNetworkSDK,
  ASSET_TYPE_REAL_ESTATE,
  ATTESTATION_STATUS_VERIFIED,
  type CustodianProfile,
  type AttestationRequest,
  type ProofData
} from '../sdk/src/custody';
import { createCustodyMonitoring, type MonitoringConfig } from '../sdk/src/custodyMonitoring';

/**
 * Real Estate Property Inspection Example
 * 
 * This example demonstrates how to verify physical real estate assets
 * using the Custody Verification Oracle system. It includes:
 * - Property deed verification
 * - Title insurance status
 * - Rental income proof
 * - Inspection reports
 * - Satellite imagery integration
 * - Multi-sig verification (3-of-5)
 */

interface Property {
  id: string;
  address: string;
  type: 'residential' | 'commercial' | 'industrial' | 'land';
  squareFootage: number;
  yearBuilt: number;
  lastSaleDate: number;
  lastSalePrice: number;
}

interface PropertyInspection {
  propertyId: string;
  inspector: string;
  inspectionDate: number;
  condition: 'excellent' | 'good' | 'fair' | 'poor';
  structuralIntegrity: 'pass' | 'fail' | 'needs-repair';
  systemsStatus: {
    hvac: 'operational' | 'needs-repair' | 'replaced';
    plumbing: 'operational' | 'needs-repair' | 'replaced';
    electrical: 'operational' | 'needs-repair' | 'replaced';
    roofing: 'good' | 'fair' | 'poor';
  };
  estimatedValue: number;
  recommendedRepairs: string[];
}

interface TitleInsurance {
  policyNumber: string;
  provider: string;
  coverageAmount: number;
  issueDate: number;
  expiryDate: number;
  status: 'active' | 'expired' | 'claims';
}

interface RentalIncome {
  propertyId: string;
  monthlyRent: number;
  occupancyRate: number;
  leaseEndDate: number;
  tenantName: string;
  paymentStatus: 'current' | 'late' | 'default';
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

async function propertyInspection() {
  console.log('🏠 Starting Real Estate Property Inspection');
  console.log('============================================\n');

  // Initialize SDK
  const custodyClient = new CustodyClient(CONFIG);
  const custodySDK = new CustodyNetworkSDK(CONFIG);

  // Step 1: Register Verified Custodians
  console.log('📋 Step 1: Registering Verified Custodians');
  console.log('-------------------------------------------');
  
  const custodians: CustodianProfile[] = [
    {
      name: 'CBRE Group',
      jurisdiction: 'United States',
      license: 'US-RE-2024-001',
      credentials: ['Licensed Real Estate Appraiser', 'IREM Certified', 'MAI Designation']
    },
    {
      name: 'JLL',
      jurisdiction: 'United Kingdom',
      license: 'UK-RE-2024-002',
      credentials: ['RICS Registered Valuer', 'IREM Certified']
    },
    {
      name: 'Cushman & Wakefield',
      jurisdiction: 'Global',
      license: 'GLOBAL-RE-2024-003',
      credentials: ['Licensed Appraiser', 'CRE Designation']
    },
    {
      name: 'Knight Frank',
      jurisdiction: 'United Kingdom',
      license: 'UK-RE-2024-004',
      credentials: ['RICS Registered Valuer', 'FRICS']
    },
    {
      name: 'CBRE Japan',
      jurisdiction: 'Japan',
      license: 'JP-RE-2024-005',
      credentials: ['Licensed Real Estate Appraiser', 'AREB']
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

  // Step 2: Property Data
  console.log('📋 Step 2: Property Information');
  console.log('---------------------------------');

  const property: Property = {
    id: 'PROP-NY-001',
    address: '350 Fifth Avenue, New York, NY 10118',
    type: 'commercial',
    squareFootage: 2768591,
    yearBuilt: 1931,
    lastSaleDate: Date.now() - (5 * 365 * 24 * 60 * 60 * 1000),
    lastSalePrice: 1100000000
  };

  console.log(`  Property ID: ${property.id}`);
  console.log(`  Address: ${property.address}`);
  console.log(`  Type: ${property.type}`);
  console.log(`  Size: ${property.squareFootage.toLocaleString()} sq ft`);
  console.log(`  Year Built: ${property.yearBuilt}`);
  console.log(`  Last Sale: $${property.lastSalePrice.toLocaleString()}`);
  console.log('');

  // Step 3: Property Inspection
  console.log('📋 Step 3: Property Inspection Report');
  console.log('---------------------------------------');

  const inspection: PropertyInspection = {
    propertyId: property.id,
    inspector: 'CBRE Group',
    inspectionDate: Date.now(),
    condition: 'good',
    structuralIntegrity: 'pass',
    systemsStatus: {
      hvac: 'operational',
      plumbing: 'operational',
      electrical: 'operational',
      roofing: 'good'
    },
    estimatedValue: 1250000000,
    recommendedRepairs: [
      'Replace elevator system (2025)',
      'Update HVAC controls to smart system',
      'Refresh lobby common areas'
    ]
  };

  console.log(`  Inspector: ${inspection.inspector}`);
  console.log(`  Date: ${new Date(inspection.inspectionDate).toISOString()}`);
  console.log(`  Condition: ${inspection.condition.toUpperCase()}`);
  console.log(`  Structural: ${inspection.structuralIntegrity.toUpperCase()}`);
  console.log(`  Estimated Value: $${inspection.estimatedValue.toLocaleString()}`);
  console.log(`  Recommended Repairs: ${inspection.recommendedRepairs.length}`);
  console.log('');

  // Step 4: Title Insurance
  console.log('📋 Step 4: Title Insurance Status');
  console.log('-----------------------------------');

  const titleInsurance: TitleInsurance = {
    policyNumber: 'TI-NY-2024-001',
    provider: 'First American Title Insurance Company',
    coverageAmount: 1250000000,
    issueDate: Date.now() - (5 * 365 * 24 * 60 * 60 * 1000),
    expiryDate: Date.now() + (5 * 365 * 24 * 60 * 60 * 1000),
    status: 'active'
  };

  console.log(`  Policy: ${titleInsurance.policyNumber}`);
  console.log(`  Provider: ${titleInsurance.provider}`);
  console.log(`  Coverage: $${titleInsurance.coverageAmount.toLocaleString()}`);
  console.log(`  Expiry: ${new Date(titleInsurance.expiryDate).toISOString()}`);
  console.log(`  Status: ${titleInsurance.status.toUpperCase()}`);
  console.log('');

  // Step 5: Rental Income
  console.log('📋 Step 5: Rental Income Verification');
  console.log('---------------------------------------');

  const rentalIncome: RentalIncome = {
    propertyId: property.id,
    monthlyRent: 8500000, // $8.5M/month
    occupancyRate: 98.5,
    leaseEndDate: Date.now() + (3 * 365 * 24 * 60 * 60 * 1000),
    tenantName: 'Various (Multi-tenant)',
    paymentStatus: 'current'
  };

  console.log(`  Monthly Rent: $${rentalIncome.monthlyRent.toLocaleString()}`);
  console.log(`  Occupancy: ${rentalIncome.occupancyRate}%`);
  console.log(`  Lease End: ${new Date(rentalIncome.leaseEndDate).toISOString()}`);
  console.log(`  Payment Status: ${rentalIncome.paymentStatus.toUpperCase()}`);
  console.log('');

  // Step 6: Generate Cryptographic Proofs
  console.log('📋 Step 6: Generating Cryptographic Proofs');
  console.log('-------------------------------------------');

  const proofData: ProofData = {
    documentHash: generateDocumentHash({
      property,
      inspection,
      titleInsurance,
      rentalIncome
    }),
    timestamp: Math.floor(Date.now() / 1000),
    satelliteData: 'SAT-IMAGERY-HASH-001',
    notarySignature: 'NOTARY_SIG_HERE'
  };

  console.log(`  Document Hash: ${proofData.documentHash}`);
  console.log(`  Timestamp: ${new Date(proofData.timestamp * 1000).toISOString()}`);
  console.log(`  Satellite Data: ${proofData.satelliteData}`);
  console.log('');

  // Step 7: Submit Attestation
  console.log('📋 Step 7: Submitting Custody Attestation');
  console.log('-------------------------------------------');

  const assetId = `PROP-${property.id}`;
  
  const conditionReport = `Condition: ${inspection.condition}, Structural: ${inspection.structuralIntegrity}, Systems: HVAC ${inspection.systemsStatus.hvac}, Plumbing ${inspection.systemsStatus.plumbing}, Electrical ${inspection.systemsStatus.electrical}, Roofing ${inspection.systemsStatus.roofing}`;
  
  const attestationRequest: AttestationRequest = {
    assetId,
    location: property.address,
    condition: conditionReport,
    value: inspection.estimatedValue.toString(),
    proofData,
    assetType: ASSET_TYPE_REAL_ESTATE
  };

  try {
    const attestationId = await custodyClient.submitAttestation(
      CUSTODIAN_KEYS[0],
      custodianIds[0] || 1,
      attestationRequest
    );
    console.log(`  ✓ Attestation submitted (ID: ${attestationId})`);
    console.log(`  Asset ID: ${assetId}`);
    console.log(`  Value: $${parseInt(attestationRequest.value).toLocaleString()}`);
    console.log('');
  } catch (error) {
    console.log(`  ⚠ Attestation submission: ${error}`);
    console.log('');
  }

  // Step 8: Multi-Sig Verification (3-of-5)
  console.log('📋 Step 8: Multi-Sig Verification (3-of-5)');
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

  // Step 9: Verify Asset Backing
  console.log('📋 Step 9: Verifying Asset Backing');
  console.log('------------------------------------');

  const isBacked = await custodyClient.verifyAssetBacking(assetId);
  console.log(`  Asset Backing Status: ${isBacked ? '✅ VERIFIED' : '❌ NOT VERIFIED'}`);
  console.log('');

  // Step 10: Get Custody History
  console.log('📋 Step 10: Custody History');
  console.log('----------------------------');

  const history = await custodyClient.getCustodyHistory(assetId);
  console.log(`  Total attestations: ${history.length}`);
  for (const att of history) {
    console.log(`  - ${new Date(att.timestamp * 1000).toISOString()}: ${att.status === ATTESTATION_STATUS_VERIFIED ? 'Verified' : 'Pending'}`);
  }
  console.log('');

  // Step 11: Set Up Monitoring
  console.log('📋 Step 11: Setting Up Monitoring');
  console.log('-----------------------------------');

  const monitoringConfig: Partial<MonitoringConfig> = {
    checkIntervalMs: 86400000, // 24 hours
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
      },
      onDepreciation: (assetId, oldValue, newValue) => {
        console.log(`  🔔 DEPRECIATION: ${assetId} - $${oldValue} → $${newValue}`);
      }
    }
  };

  const monitoring = createCustodyMonitoring(custodyClient, monitoringConfig);
  monitoring.trackAsset(assetId);
  monitoring.start();

  console.log('  ✓ Monitoring started');
  console.log('  ✓ Tracking asset:', assetId);
  console.log('');

  // Step 12: Insurance Integration
  console.log('📋 Step 12: Insurance Status Check');
  console.log('------------------------------------');

  const insuranceStatus = {
    provider: 'Berkley Insurance',
    policyNumber: 'PROP-2024-NY-001',
    coverageAmount: '1250000000',
    expiryDate: Date.now() + (365 * 24 * 60 * 60 * 1000),
    verified: true
  };

  console.log(`  Provider: ${insuranceStatus.provider}`);
  console.log(`  Policy: ${insuranceStatus.policyNumber}`);
  console.log(`  Coverage: $${parseInt(insuranceStatus.coverageAmount).toLocaleString()}`);
  console.log(`  Expiry: ${new Date(insuranceStatus.expiryDate).toISOString()}`);
  console.log(`  Status: ${insuranceStatus.verified ? '✅ VERIFIED' : '❌ NOT VERIFIED'}`);
  console.log('');

  // Step 13: Summary
  console.log('📋 Summary');
  console.log('----------');
  console.log(`  ✅ Custodians Registered: ${custodianIds.length}`);
  console.log(`  ✅ Property: ${property.address}`);
  console.log(`  ✅ Inspection: ${inspection.condition.toUpperCase()}`);
  console.log(`  ✅ Title Insurance: ${titleInsurance.status.toUpperCase()}`);
  console.log(`  ✅ Rental Income: $${rentalIncome.monthlyRent.toLocaleString()}/month`);
  console.log(`  ✅ Attestation Submitted: ${assetId}`);
  console.log(`  ✅ Multi-Sig Verification: 3-of-5`);
  console.log(`  ✅ Asset Backing: ${isBacked ? 'VERIFIED' : 'PENDING'}`);
  console.log(`  ✅ Insurance: Active`);
  console.log(`  ✅ Monitoring: Active`);
  console.log('');
  console.log('🎉 Property Inspection Complete!');
  console.log('==================================\n');

  // Cleanup
  setTimeout(() => {
    monitoring.stop();
    console.log('Monitoring stopped.');
  }, 5000);
}

/**
 * Generate document hash (simplified)
 */
function generateDocumentHash(data: {
  property: Property;
  inspection: PropertyInspection;
  titleInsurance: TitleInsurance;
  rentalIncome: RentalIncome;
}): string {
  const jsonData = JSON.stringify({
    property: { id: data.property.id, address: data.property.address },
    inspection: { condition: data.inspection.condition, estimatedValue: data.inspection.estimatedValue },
    titleInsurance: { policyNumber: data.titleInsurance.policyNumber, status: data.titleInsurance.status },
    rentalIncome: { monthlyRent: data.rentalIncome.monthlyRent, occupancyRate: data.rentalIncome.occupancyRate }
  });
  
  // Simple hash - use proper crypto in production
  let hash = 0;
  for (let i = 0; i < jsonData.length; i++) {
    const char = jsonData.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash).toString(16).padStart(16, '0');
}

// Run the example
propertyInspection().catch(console.error);

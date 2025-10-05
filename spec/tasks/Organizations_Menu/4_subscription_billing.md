# Task 4: Subscription Billing System

## Implementation Details

This task implements a comprehensive subscription billing system for organizations, supporting tiered pricing, seat management, usage tracking, and enterprise billing workflows with real-time cost optimization.

### Architecture Overview

The system uses Bevy ECS with time-based billing cycles, event-driven usage tracking, and automated billing workflows to provide enterprise-grade subscription management with flexible pricing models.

### Core Billing Data Structures

#### Subscription Management System
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, NaiveDate};
use uuid::Uuid;
use std::collections::{HashMap, BTreeMap, VecDeque};
use rust_decimal::Decimal;

/// Comprehensive subscription billing system
/// References: docs/bevy/examples/time/* (billing cycle timing and scheduling)
/// References: docs/bevy/examples/ecs/event.rs (billing event processing)
#[derive(Resource, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct SubscriptionBillingSystem {
    /// Active subscriptions by organization
    pub active_subscriptions: HashMap<Uuid, OrganizationSubscription>,
    /// Billing cycle management
    pub billing_cycles: BillingCycleManager,
    /// Usage tracking and metering
    pub usage_tracker: UsageTracker,
    /// Payment processing integration
    pub payment_processor: PaymentProcessor,
    /// Invoice generation and management
    pub invoice_manager: InvoiceManager,
    /// Pricing engine and calculations
    pub pricing_engine: PricingEngine,
    /// Seat management and optimization
    pub seat_manager: SeatManager,
}

/// Detailed organization subscription
/// References: docs/bevy/examples/time/time.rs (billing timing)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationSubscription {
    /// Subscription unique identifier
    pub subscription_id: Uuid,
    /// Associated organization
    pub organization_id: Uuid,
    /// Current subscription plan
    pub current_plan: SubscriptionPlan,
    /// Plan change history
    pub plan_history: Vec<PlanChangeRecord>,
    /// Subscription lifecycle status
    pub status: SubscriptionStatus,
    /// Billing configuration
    pub billing_config: BillingConfiguration,
    /// Current billing period
    pub current_period: BillingPeriod,
    /// Next billing period
    pub next_period: Option<BillingPeriod>,
    /// Seat allocation and management
    pub seat_allocation: SeatAllocation,
    /// Usage limits and quotas
    pub usage_quotas: UsageQuotas,
    /// Billing history and records
    pub billing_history: BillingHistory,
    /// Payment method and details
    pub payment_details: PaymentDetails,
    /// Subscription preferences
    pub preferences: SubscriptionPreferences,
}

/// Comprehensive subscription plans with pricing
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
pub enum SubscriptionPlan {
    /// Free tier with basic features
    Free {
        /// Maximum team members
        max_members: usize,
        /// Feature limitations
        feature_limits: FeatureLimits,
        /// Usage quotas
        usage_quotas: FreeUsageQuotas,
    },
    /// Professional plan for growing teams
    Professional {
        /// Base monthly cost per seat
        cost_per_seat: Decimal,
        /// Minimum seats required
        minimum_seats: usize,
        /// Professional feature set
        features: ProfessionalFeatures,
        /// Enhanced usage quotas
        usage_quotas: ProfessionalUsageQuotas,
        /// Volume discounts
        volume_discounts: Vec<VolumeDiscount>,
    },
    /// Enterprise plan for large organizations
    Enterprise {
        /// Custom pricing structure
        pricing_structure: EnterprisePricing,
        /// Enterprise feature set
        features: EnterpriseFeatures,
        /// Unlimited or high usage quotas
        usage_quotas: EnterpriseUsageQuotas,
        /// Service level agreements
        sla_terms: ServiceLevelAgreements,
        /// Dedicated support tier
        support_tier: SupportTier,
    },
    /// Custom negotiated plan
    Custom {
        /// Contract identifier
        contract_id: String,
        /// Custom pricing details
        pricing_details: CustomPricingDetails,
        /// Negotiated features
        negotiated_features: NegotiatedFeatures,
        /// Contract terms and conditions
        contract_terms: ContractTerms,
        /// Renewal terms
        renewal_terms: RenewalTerms,
    },
}

/// Seat allocation and management
/// References: docs/bevy/examples/ecs/change_detection.rs (seat usage tracking)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SeatAllocation {
    /// Total seats purchased
    pub total_seats: usize,
    /// Currently occupied seats
    pub occupied_seats: usize,
    /// Available seats for allocation
    pub available_seats: usize,
    /// Seat reservations (pending invitations)
    pub reserved_seats: usize,
    /// Seat usage history
    pub usage_history: Vec<SeatUsageSnapshot>,
    /// Auto-scaling configuration
    pub auto_scaling: Option<AutoScalingConfig>,
    /// Seat optimization settings
    pub optimization: SeatOptimization,
    /// Cost allocation per seat
    pub cost_allocation: Vec<SeatCostAllocation>,
}

/// Automated seat scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AutoScalingConfig {
    /// Enable automatic seat scaling
    pub enabled: bool,
    /// Minimum seats to maintain
    pub min_seats: usize,
    /// Maximum seats allowed
    pub max_seats: usize,
    /// Scale up threshold (percentage)
    pub scale_up_threshold: f32,
    /// Scale down threshold (percentage)
    pub scale_down_threshold: f32,
    /// Scaling cooldown period
    pub cooldown_period: Duration,
    /// Cost control limits
    pub cost_limits: CostLimits,
}
```

#### Usage Tracking and Metering
```rust
/// Comprehensive usage tracking system
/// References: docs/bevy/examples/ecs/event.rs (usage event tracking)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UsageTracker {
    /// Real-time usage monitoring
    pub usage_monitors: HashMap<Uuid, UsageMonitor>,
    /// Usage aggregation and reporting
    pub usage_aggregator: UsageAggregator,
    /// Usage-based billing calculations
    pub usage_billing: UsageBillingCalculator,
    /// Usage analytics and insights
    pub usage_analytics: UsageAnalytics,
    /// Usage quota enforcement
    pub quota_enforcer: QuotaEnforcer,
}

/// Organization usage monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UsageMonitor {
    /// Organization being monitored
    pub organization_id: Uuid,
    /// Current billing period metrics
    pub current_period_usage: PeriodUsage,
    /// Historical usage patterns
    pub usage_history: BTreeMap<NaiveDate, DailyUsage>,
    /// Real-time usage counters
    pub real_time_counters: UsageCounters,
    /// Usage trend analysis
    pub trend_analysis: UsageTrendAnalysis,
}

/// Detailed usage metrics per billing period
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct PeriodUsage {
    /// API calls made this period
    pub api_calls: u64,
    /// Data storage consumed (bytes)
    pub storage_bytes: u64,
    /// Data transfer (bytes)
    pub data_transfer_bytes: u64,
    /// Extension installations
    pub extension_installations: u32,
    /// Build minutes consumed
    pub build_minutes: u32,
    /// Support ticket count
    pub support_tickets: u32,
    /// Custom usage metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Per-member usage breakdown
    pub per_member_usage: HashMap<Uuid, MemberUsageBreakdown>,
}

/// Real-time usage counters
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct UsageCounters {
    /// Counters by category
    pub counters: HashMap<UsageCategory, u64>,
    /// Counter update timestamps
    pub last_updated: HashMap<UsageCategory, DateTime<Utc>>,
    /// Counter reset schedule
    pub reset_schedule: CounterResetSchedule,
}

/// Usage categories for billing
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum UsageCategory {
    /// API request usage
    ApiRequests,
    /// Data storage usage
    Storage,
    /// Bandwidth usage
    Bandwidth,
    /// Compute usage (CI/CD)
    Compute,
    /// Extension store usage
    ExtensionStore,
    /// Support and professional services
    Support,
    /// Custom usage category
    Custom(String),
}
```

#### Billing Cycle Management
```rust
/// Billing cycle management system
/// References: docs/bevy/examples/time/time.rs (billing cycle timing)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BillingCycleManager {
    /// Active billing cycles
    pub active_cycles: HashMap<Uuid, BillingCycle>,
    /// Billing schedule management
    pub schedule_manager: BillingScheduleManager,
    /// Prorated billing calculations
    pub proration_calculator: ProrationCalculator,
    /// Billing event queue
    pub event_queue: VecDeque<BillingEvent>,
    /// Retry and failure handling
    pub retry_handler: BillingRetryHandler,
}

/// Individual billing cycle
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BillingCycle {
    /// Cycle unique identifier
    pub cycle_id: Uuid,
    /// Organization subscription
    pub subscription_id: Uuid,
    /// Billing period start
    pub period_start: DateTime<Utc>,
    /// Billing period end
    pub period_end: DateTime<Utc>,
    /// Cycle status
    pub status: BillingCycleStatus,
    /// Usage during this cycle
    pub cycle_usage: PeriodUsage,
    /// Charges for this cycle
    pub cycle_charges: Vec<BillingCharge>,
    /// Total amount for cycle
    pub total_amount: Decimal,
    /// Currency for billing
    pub currency: String,
    /// Invoice generation details
    pub invoice_details: Option<InvoiceGenerationDetails>,
}

/// Types of billing charges
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BillingCharge {
    /// Charge unique identifier
    pub charge_id: Uuid,
    /// Charge type and category
    pub charge_type: ChargeType,
    /// Charge description
    pub description: String,
    /// Quantity or usage amount
    pub quantity: f64,
    /// Unit price
    pub unit_price: Decimal,
    /// Total charge amount
    pub total_amount: Decimal,
    /// Proration information
    pub proration_info: Option<ProrationInfo>,
    /// Applicable discounts
    pub discounts: Vec<AppliedDiscount>,
}

/// Different types of charges
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ChargeType {
    /// Subscription base fee
    SubscriptionFee {
        plan_type: String,
        billing_frequency: BillingFrequency,
    },
    /// Per-seat charges
    SeatCharge {
        seat_count: usize,
        seat_tier: SeatTier,
    },
    /// Usage-based charges
    UsageCharge {
        usage_category: UsageCategory,
        usage_amount: f64,
        pricing_model: UsagePricingModel,
    },
    /// One-time fees
    OneTimeFee {
        fee_type: String,
        reason: String,
    },
    /// Credits and adjustments
    Credit {
        credit_type: CreditType,
        reason: String,
    },
    /// Taxes and fees
    Tax {
        tax_type: TaxType,
        tax_rate: f32,
        taxable_amount: Decimal,
    },
}
```

#### Payment Processing Integration
```rust
/// Payment processing system
/// References: docs/bevy/examples/ecs/event.rs (payment event handling)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PaymentProcessor {
    /// Payment gateway integrations
    pub payment_gateways: Vec<PaymentGateway>,
    /// Payment method management
    pub payment_methods: HashMap<Uuid, PaymentMethod>,
    /// Transaction processing
    pub transaction_processor: TransactionProcessor,
    /// Payment security and compliance
    pub security_manager: PaymentSecurityManager,
    /// Payment retry and recovery
    pub retry_manager: PaymentRetryManager,
}

/// Payment gateway integration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PaymentGateway {
    /// Gateway provider name
    pub provider: PaymentProvider,
    /// Gateway configuration
    pub config: GatewayConfig,
    /// Supported features
    pub features: PaymentFeatures,
    /// Processing fees
    pub processing_fees: ProcessingFees,
    /// Gateway status
    pub status: GatewayStatus,
}

/// Payment method details
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PaymentMethod {
    /// Payment method ID
    pub method_id: Uuid,
    /// Organization owner
    pub organization_id: Uuid,
    /// Payment method type
    pub method_type: PaymentMethodType,
    /// Display information
    pub display_info: PaymentMethodDisplay,
    /// Method status and verification
    pub status: PaymentMethodStatus,
    /// Security and tokenization
    pub security_info: PaymentMethodSecurity,
    /// Auto-payment configuration
    pub auto_payment: AutoPaymentConfig,
    /// Payment method preferences
    pub preferences: PaymentMethodPreferences,
}

/// Different types of payment methods
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum PaymentMethodType {
    /// Credit or debit card
    CreditCard {
        /// Tokenized card information
        token: String,
        /// Card brand
        brand: CardBrand,
        /// Last four digits
        last_four: String,
        /// Expiration month/year
        expiry: CardExpiry,
        /// Billing address
        billing_address: BillingAddress,
    },
    /// Bank account (ACH)
    BankAccount {
        /// Account token
        token: String,
        /// Account type
        account_type: BankAccountType,
        /// Bank name
        bank_name: String,
        /// Last four digits of account
        last_four: String,
    },
    /// Wire transfer
    WireTransfer {
        /// Bank details
        bank_details: WireTransferDetails,
        /// Transfer instructions
        instructions: String,
    },
    /// Purchase order
    PurchaseOrder {
        /// PO number
        po_number: String,
        /// PO amount limit
        amount_limit: Option<Decimal>,
        /// Approval workflow
        approval_workflow: POApprovalWorkflow,
    },
}
```

#### Invoice Management System
```rust
/// Comprehensive invoice management
/// References: docs/bevy/examples/asset/* (invoice PDF generation)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct InvoiceManager {
    /// Generated invoices
    pub invoices: HashMap<Uuid, Invoice>,
    /// Invoice templates and customization
    pub templates: InvoiceTemplateManager,
    /// Invoice generation engine
    pub generation_engine: InvoiceGenerationEngine,
    /// Invoice delivery system
    pub delivery_system: InvoiceDeliverySystem,
    /// Payment tracking and reconciliation
    pub payment_tracker: PaymentTracker,
}

/// Detailed invoice information
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Invoice {
    /// Invoice unique identifier
    pub invoice_id: Uuid,
    /// Invoice number (human-readable)
    pub invoice_number: String,
    /// Organization being billed
    pub organization_id: Uuid,
    /// Billing period covered
    pub billing_period: BillingPeriod,
    /// Invoice generation date
    pub generated_at: DateTime<Utc>,
    /// Invoice due date
    pub due_date: DateTime<Utc>,
    /// Invoice status
    pub status: InvoiceStatus,
    /// Line items and charges
    pub line_items: Vec<InvoiceLineItem>,
    /// Subtotal before taxes
    pub subtotal: Decimal,
    /// Applied taxes
    pub taxes: Vec<TaxLineItem>,
    /// Total invoice amount
    pub total_amount: Decimal,
    /// Currency
    pub currency: String,
    /// Payment terms
    pub payment_terms: PaymentTerms,
    /// Invoice customization
    pub customization: InvoiceCustomization,
    /// Delivery information
    pub delivery_info: InvoiceDeliveryInfo,
}

/// Individual invoice line item
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct InvoiceLineItem {
    /// Line item ID
    pub item_id: Uuid,
    /// Item description
    pub description: String,
    /// Item category
    pub category: InvoiceItemCategory,
    /// Quantity
    pub quantity: f64,
    /// Unit price
    pub unit_price: Decimal,
    /// Line total
    pub line_total: Decimal,
    /// Usage period (if applicable)
    pub usage_period: Option<DateRange>,
    /// Proration details
    pub proration: Option<ProrationDetails>,
    /// Related billing charge
    pub charge_id: Option<Uuid>,
}
```

#### Pricing Engine and Calculations
```rust
/// Advanced pricing engine
/// References: docs/bevy/examples/ecs/system_param.rs (pricing calculation systems)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PricingEngine {
    /// Pricing models and strategies
    pub pricing_models: HashMap<String, PricingModel>,
    /// Discount and promotion engine
    pub discount_engine: DiscountEngine,
    /// Tax calculation system
    pub tax_calculator: TaxCalculator,
    /// Currency conversion
    pub currency_converter: CurrencyConverter,
    /// Pricing optimization
    pub optimization_engine: PricingOptimizationEngine,
}

/// Flexible pricing model
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum PricingModel {
    /// Fixed price per billing cycle
    Fixed {
        amount: Decimal,
        currency: String,
    },
    /// Per-seat pricing
    PerSeat {
        base_price: Decimal,
        minimum_seats: usize,
        volume_tiers: Vec<VolumeDiscountTier>,
    },
    /// Usage-based pricing
    UsageBased {
        base_fee: Option<Decimal>,
        usage_tiers: Vec<UsageTier>,
        overage_rate: Decimal,
    },
    /// Hybrid pricing (fixed + usage)
    Hybrid {
        fixed_component: Decimal,
        usage_components: Vec<UsageComponent>,
    },
    /// Custom pricing formula
    Custom {
        formula: PricingFormula,
        variables: HashMap<String, f64>,
    },
}

/// Usage-based pricing tier
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UsageTier {
    /// Tier name/description
    pub tier_name: String,
    /// Usage range for this tier
    pub usage_range: UsageRange,
    /// Price per unit in this tier
    pub price_per_unit: Decimal,
    /// Tier discount (if applicable)
    pub discount: Option<TierDiscount>,
}

/// Discount and promotion engine
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct DiscountEngine {
    /// Available discounts
    pub available_discounts: Vec<Discount>,
    /// Promotion campaigns
    pub active_promotions: Vec<Promotion>,
    /// Discount validation rules
    pub validation_rules: DiscountValidationRules,
    /// Discount application order
    pub application_order: DiscountApplicationOrder,
}
```

### System Implementation

#### Billing Cycle Processing Systems
```rust
/// Billing cycle processing systems
/// References: docs/bevy/examples/time/time.rs (time-based billing processing)

/// Main billing cycle processing system
pub fn billing_cycle_processing_system(
    mut billing_system: ResMut<SubscriptionBillingSystem>,
    mut billing_events: EventWriter<BillingEvent>,
    time: Res<Time>,
) {
    let current_time = Utc::now();
    
    // Process billing cycles that are ready
    for (org_id, subscription) in &billing_system.active_subscriptions {
        if let Some(current_period) = &subscription.current_period {
            // Check if billing cycle should end
            if current_time >= current_period.end_date {
                // Generate invoice for completed period
                let invoice_result = generate_period_invoice(
                    *org_id,
                    subscription,
                    &billing_system.usage_tracker,
                    &billing_system.pricing_engine,
                );
                
                match invoice_result {
                    Ok(invoice) => {
                        // Add invoice to manager
                        billing_system.invoice_manager.invoices.insert(invoice.invoice_id, invoice.clone());
                        
                        // Send billing event
                        billing_events.send(BillingEvent::InvoiceGenerated {
                            invoice_id: invoice.invoice_id,
                            organization_id: *org_id,
                            amount: invoice.total_amount,
                        });
                        
                        // Start next billing cycle
                        start_next_billing_cycle(*org_id, subscription, &mut billing_system);
                    },
                    Err(error) => {
                        // Handle billing error
                        billing_events.send(BillingEvent::BillingError {
                            organization_id: *org_id,
                            error_type: BillingErrorType::InvoiceGeneration,
                            error_message: error.to_string(),
                        });
                    }
                }
            }
            
            // Check for mid-cycle plan changes
            check_plan_changes(*org_id, subscription, &mut billing_system, &mut billing_events);
        }
    }
    
    // Process usage tracking updates
    update_usage_tracking(&mut billing_system.usage_tracker, current_time);
    
    // Check seat auto-scaling
    process_auto_scaling(&mut billing_system.seat_manager, &mut billing_events);
}

/// Usage tracking system
/// References: docs/bevy/examples/ecs/event.rs (usage event processing)
pub fn usage_tracking_system(
    mut usage_tracker: ResMut<UsageTracker>,
    mut usage_events: EventReader<UsageEvent>,
    mut billing_events: EventWriter<BillingEvent>,
) {
    for event in usage_events.read() {
        match event {
            UsageEvent::ApiCallMade { organization_id, endpoint, cost_weight } => {
                if let Some(monitor) = usage_tracker.usage_monitors.get_mut(organization_id) {
                    // Increment API call counter
                    monitor.current_period_usage.api_calls += 1;
                    monitor.real_time_counters.counters
                        .entry(UsageCategory::ApiRequests)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                    
                    // Apply cost weighting for expensive operations
                    let weighted_usage = cost_weight.unwrap_or(1.0);
                    update_usage_metrics(*organization_id, UsageCategory::ApiRequests, weighted_usage, &mut usage_tracker);
                    
                    // Check quota limits
                    if let Some(quota_violation) = check_quota_violation(*organization_id, UsageCategory::ApiRequests, &usage_tracker) {
                        billing_events.send(BillingEvent::QuotaExceeded {
                            organization_id: *organization_id,
                            quota_type: UsageCategory::ApiRequests,
                            current_usage: quota_violation.current_usage,
                            quota_limit: quota_violation.quota_limit,
                        });
                    }
                }
            },
            UsageEvent::StorageUsed { organization_id, bytes_used, storage_type } => {
                // Track storage usage
                track_storage_usage(*organization_id, *bytes_used, storage_type, &mut usage_tracker);
            },
            UsageEvent::DataTransfer { organization_id, bytes_transferred, transfer_type } => {
                // Track data transfer
                track_data_transfer(*organization_id, *bytes_transferred, transfer_type, &mut usage_tracker);
            },
        }
    }
    
    // Aggregate usage metrics for billing
    aggregate_usage_metrics(&mut usage_tracker);
}

/// Payment processing system
pub fn payment_processing_system(
    mut payment_processor: ResMut<PaymentProcessor>,
    mut payment_events: EventReader<PaymentEvent>,
    mut billing_events: EventWriter<BillingEvent>,
) {
    for event in payment_events.read() {
        match event {
            PaymentEvent::PaymentInitiated { payment_id, invoice_id, amount } => {
                // Process payment through gateway
                let payment_result = process_payment_transaction(
                    *payment_id,
                    *amount,
                    &payment_processor
                );
                
                match payment_result {
                    Ok(transaction) => {
                        // Payment successful
                        billing_events.send(BillingEvent::PaymentSuccessful {
                            payment_id: *payment_id,
                            invoice_id: *invoice_id,
                            amount: *amount,
                            transaction_id: transaction.transaction_id,
                        });
                        
                        // Update invoice status
                        update_invoice_payment_status(*invoice_id, PaymentStatus::Paid, &mut payment_processor);
                    },
                    Err(error) => {
                        // Payment failed
                        billing_events.send(BillingEvent::PaymentFailed {
                            payment_id: *payment_id,
                            invoice_id: *invoice_id,
                            error_code: error.code,
                            error_message: error.message,
                        });
                        
                        // Schedule retry if applicable
                        schedule_payment_retry(*payment_id, &error, &mut payment_processor);
                    }
                }
            },
            PaymentEvent::PaymentMethodAdded { organization_id, payment_method } => {
                // Validate and store payment method
                validate_and_store_payment_method(*organization_id, payment_method, &mut payment_processor);
            },
            PaymentEvent::PaymentMethodUpdated { method_id, updates } => {
                // Update payment method
                update_payment_method(*method_id, updates, &mut payment_processor);
            },
        }
    }
    
    // Process payment retries
    process_payment_retries(&mut payment_processor, &mut billing_events);
}

/// Seat management system
/// References: docs/bevy/examples/ecs/change_detection.rs (seat allocation tracking)
pub fn seat_management_system(
    mut seat_manager: ResMut<SeatManager>,
    mut seat_events: EventReader<SeatManagementEvent>,
    mut billing_events: EventWriter<BillingEvent>,
    billing_system: Res<SubscriptionBillingSystem>,
) {
    for event in seat_events.read() {
        match event {
            SeatManagementEvent::SeatAllocated { organization_id, member_id } => {
                if let Some(subscription) = billing_system.active_subscriptions.get(organization_id) {
                    let mut seat_allocation = subscription.seat_allocation.clone();
                    
                    // Allocate seat to member
                    if seat_allocation.available_seats > 0 {
                        seat_allocation.occupied_seats += 1;
                        seat_allocation.available_seats -= 1;
                        
                        // Record seat allocation
                        let allocation_record = SeatAllocationRecord {
                            member_id: *member_id,
                            allocated_at: Utc::now(),
                            seat_tier: determine_seat_tier(*member_id, subscription),
                            cost_allocation: calculate_seat_cost(*member_id, subscription),
                        };
                        
                        seat_manager.allocation_records.insert(*member_id, allocation_record);
                        
                        billing_events.send(BillingEvent::SeatAllocated {
                            organization_id: *organization_id,
                            member_id: *member_id,
                            seat_cost: allocation_record.cost_allocation,
                        });
                    } else {
                        // Check auto-scaling
                        if let Some(auto_scaling) = &seat_allocation.auto_scaling {
                            if auto_scaling.enabled && seat_allocation.total_seats < auto_scaling.max_seats {
                                // Auto-scale up
                                trigger_seat_auto_scaling(*organization_id, ScalingDirection::Up, &mut seat_manager);
                            }
                        }
                    }
                }
            },
            SeatManagementEvent::SeatDeallocated { organization_id, member_id } => {
                // Handle seat deallocation
                deallocate_member_seat(*organization_id, *member_id, &mut seat_manager, &mut billing_events);
            },
            SeatManagementEvent::SeatUpgraded { organization_id, member_id, new_tier } => {
                // Handle seat tier upgrade
                upgrade_member_seat(*organization_id, *member_id, new_tier, &mut seat_manager, &mut billing_events);
            },
        }
    }
    
    // Optimize seat allocation
    optimize_seat_allocation(&mut seat_manager, &billing_system);
}
```

### Event System Integration

```rust
/// Billing system events
/// References: docs/bevy/examples/ecs/event.rs (billing event patterns)

#[derive(Event, Debug, Clone)]
pub enum BillingEvent {
    /// Invoice generated
    InvoiceGenerated {
        invoice_id: Uuid,
        organization_id: Uuid,
        amount: Decimal,
    },
    /// Payment processed successfully
    PaymentSuccessful {
        payment_id: Uuid,
        invoice_id: Uuid,
        amount: Decimal,
        transaction_id: String,
    },
    /// Payment failed
    PaymentFailed {
        payment_id: Uuid,
        invoice_id: Uuid,
        error_code: String,
        error_message: String,
    },
    /// Usage quota exceeded
    QuotaExceeded {
        organization_id: Uuid,
        quota_type: UsageCategory,
        current_usage: f64,
        quota_limit: f64,
    },
    /// Seat allocated to member
    SeatAllocated {
        organization_id: Uuid,
        member_id: Uuid,
        seat_cost: Decimal,
    },
    /// Billing error occurred
    BillingError {
        organization_id: Uuid,
        error_type: BillingErrorType,
        error_message: String,
    },
}

#[derive(Event, Debug, Clone)]
pub enum UsageEvent {
    /// API call made
    ApiCallMade {
        organization_id: Uuid,
        endpoint: String,
        cost_weight: Option<f64>,
    },
    /// Storage used
    StorageUsed {
        organization_id: Uuid,
        bytes_used: u64,
        storage_type: StorageType,
    },
    /// Data transferred
    DataTransfer {
        organization_id: Uuid,
        bytes_transferred: u64,
        transfer_type: TransferType,
    },
}

#[derive(Event, Debug, Clone)]
pub enum SeatManagementEvent {
    /// Seat allocated to member
    SeatAllocated {
        organization_id: Uuid,
        member_id: Uuid,
    },
    /// Seat deallocated from member
    SeatDeallocated {
        organization_id: Uuid,
        member_id: Uuid,
    },
    /// Seat upgraded to higher tier
    SeatUpgraded {
        organization_id: Uuid,
        member_id: Uuid,
        new_tier: SeatTier,
    },
}
```

### Plugin Registration

```rust
/// Subscription billing plugin for Bevy integration
/// References: docs/bevy/examples/app/* (plugin patterns)
pub struct SubscriptionBillingPlugin;

impl Plugin for SubscriptionBillingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SubscriptionBillingSystem>()
            .add_event::<BillingEvent>()
            .add_event::<UsageEvent>()
            .add_event::<PaymentEvent>()
            .add_event::<SeatManagementEvent>()
            .add_systems(
                Update,
                (
                    billing_cycle_processing_system,
                    usage_tracking_system,
                    payment_processing_system,
                    seat_management_system,
                    invoice_delivery_system,
                ).chain()
            );
    }
}
```

### Implementation Requirements

1. **Multi-Currency Support**: Handle international billing with accurate currency conversion
2. **Proration Calculations**: Accurate proration for mid-cycle plan changes and seat modifications
3. **Usage-Based Billing**: Real-time usage tracking with flexible pricing models
4. **Payment Security**: PCI-compliant payment processing with tokenization
5. **Auto-Scaling Optimization**: Intelligent seat scaling based on usage patterns
6. **Tax Compliance**: Automated tax calculation for global billing requirements
7. **Enterprise Integration**: Support for purchase orders and complex approval workflows
8. **Performance Optimization**: Handle high-volume usage tracking and billing calculations

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

**Implementation References:**
- `docs/bevy/examples/time/time.rs:1-200` - Time-based billing cycle management
- `docs/bevy/examples/ecs/event.rs:1-144` - Event-driven billing and payment processing
- `docs/bevy/examples/ecs/system_param.rs:1-180` - System parameter patterns for billing calculations
- `docs/bevy/examples/ecs/change_detection.rs:1-106` - Usage and seat allocation change tracking
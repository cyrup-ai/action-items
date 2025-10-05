# Task 5: QA Subscription Billing System Validation

## Implementation Details

**Act as an Objective QA Rust Developer** and thoroughly validate the subscription billing system implementation from Task 4, ensuring accurate financial calculations, secure payment processing, and compliance with accounting standards.

### QA Validation Overview

This task provides comprehensive quality assurance for all billing system components, validating pricing accuracy, payment security, usage tracking precision, and regulatory compliance for enterprise billing requirements.

### Billing System Validation Suite

#### Financial Accuracy Validation Tests
```rust
use bevy::prelude::*;
use chrono::{DateTime, Utc, Duration, NaiveDate};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[cfg(test)]
mod subscription_billing_qa_tests {
    use super::*;
    use crate::organizations::billing::*;
    
    /// Test subscription pricing calculations and accuracy
    /// References: docs/bevy/examples/time/time.rs (billing cycle validation)
    #[test]
    fn test_subscription_pricing_accuracy() {
        let mut app = create_test_app_with_billing_system();
        
        // Test Professional plan pricing
        let professional_plan = SubscriptionPlan::Professional {
            cost_per_seat: Decimal::from_str("15.99").unwrap(),
            minimum_seats: 5,
            features: ProfessionalFeatures::default(),
            usage_quotas: ProfessionalUsageQuotas::default(),
            volume_discounts: vec![
                VolumeDiscount {
                    threshold: 10,
                    discount_percentage: Decimal::from_str("0.10").unwrap(),
                },
                VolumeDiscount {
                    threshold: 25,
                    discount_percentage: Decimal::from_str("0.20").unwrap(),
                },
            ],
        };
        
        // Test minimum seat billing
        let minimum_cost = calculate_plan_cost(&professional_plan, 3); // Below minimum
        let expected_minimum = Decimal::from_str("15.99").unwrap() * Decimal::from(5);
        assert_eq!(minimum_cost, expected_minimum, "Minimum seats must be enforced in billing");
        
        // Test exact minimum billing
        let exact_minimum_cost = calculate_plan_cost(&professional_plan, 5);
        assert_eq!(exact_minimum_cost, expected_minimum, "Exact minimum seats must calculate correctly");
        
        // Test volume discount application (10 seats = 10% discount)
        let volume_cost_10 = calculate_plan_cost(&professional_plan, 10);
        let expected_volume_10 = Decimal::from_str("15.99").unwrap() * Decimal::from(10) * Decimal::from_str("0.90").unwrap();
        assert_eq!(volume_cost_10, expected_volume_10, "10-seat volume discount must apply correctly");
        
        // Test higher volume discount (25 seats = 20% discount)
        let volume_cost_25 = calculate_plan_cost(&professional_plan, 25);
        let expected_volume_25 = Decimal::from_str("15.99").unwrap() * Decimal::from(25) * Decimal::from_str("0.80").unwrap();
        assert_eq!(volume_cost_25, expected_volume_25, "25-seat volume discount must apply correctly");
        
        // Test Enterprise custom pricing
        let enterprise_plan = SubscriptionPlan::Enterprise {
            pricing_structure: EnterprisePricing::Custom {
                base_fee: Decimal::from_str("1000.00").unwrap(),
                seat_tiers: vec![
                    EnterpriseSeatTier {
                        min_seats: 1,
                        max_seats: 50,
                        cost_per_seat: Decimal::from_str("12.99").unwrap(),
                    },
                    EnterpriseSeatTier {
                        min_seats: 51,
                        max_seats: 200,
                        cost_per_seat: Decimal::from_str("10.99").unwrap(),
                    },
                ],
            },
            features: EnterpriseFeatures::default(),
            usage_quotas: EnterpriseUsageQuotas::default(),
            sla_terms: ServiceLevelAgreements::default(),
            support_tier: SupportTier::Dedicated,
        };
        
        // Test enterprise tiered pricing (75 seats = 50 * $12.99 + 25 * $10.99 + $1000 base)
        let enterprise_cost = calculate_plan_cost(&enterprise_plan, 75);
        let expected_enterprise = Decimal::from_str("1000.00").unwrap()  // Base fee
            + (Decimal::from_str("12.99").unwrap() * Decimal::from(50))   // First tier
            + (Decimal::from_str("10.99").unwrap() * Decimal::from(25));  // Second tier
        assert_eq!(enterprise_cost, expected_enterprise, "Enterprise tiered pricing must calculate correctly");
        
        println!("✅ Subscription pricing accuracy validation passed");
    }
    
    /// Test proration calculations for mid-cycle changes
    /// References: docs/bevy/examples/time/time.rs (time-based proration)
    #[test]
    fn test_proration_accuracy() {
        let mut app = create_test_app_with_billing_system();
        
        // Setup billing cycle (monthly)
        let cycle_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let cycle_end = cycle_start + Duration::days(30);
        let change_date = cycle_start + Duration::days(15); // Mid-cycle
        
        let billing_cycle = BillingCycle {
            cycle_id: Uuid::new_v4(),
            subscription_id: Uuid::new_v4(),
            period_start: cycle_start,
            period_end: cycle_end,
            status: BillingCycleStatus::Active,
            cycle_usage: PeriodUsage::default(),
            cycle_charges: vec![],
            total_amount: Decimal::ZERO,
            currency: "USD".to_string(),
            invoice_details: None,
        };
        
        // Test seat addition proration (adding 5 seats mid-cycle)
        let seat_cost_per_month = Decimal::from_str("15.99").unwrap();
        let prorated_cost = calculate_seat_addition_proration(
            5,
            seat_cost_per_month,
            change_date,
            &billing_cycle,
        );
        
        // Should be: 5 seats * $15.99 * (15 days remaining / 30 days total)
        let expected_proration = seat_cost_per_month * Decimal::from(5) * Decimal::from_str("0.5").unwrap();
        assert_eq!(prorated_cost, expected_proration, "Seat addition proration must be accurate");
        
        // Test plan upgrade proration
        let old_plan_monthly = Decimal::from_str("99.00").unwrap();
        let new_plan_monthly = Decimal::from_str("199.00").unwrap();
        let upgrade_proration = calculate_plan_upgrade_proration(
            old_plan_monthly,
            new_plan_monthly,
            change_date,
            &billing_cycle,
        );
        
        // Should be: ($199 - $99) * (15 days remaining / 30 days total)
        let plan_difference = new_plan_monthly - old_plan_monthly;
        let expected_upgrade = plan_difference * Decimal::from_str("0.5").unwrap();
        assert_eq!(upgrade_proration, expected_upgrade, "Plan upgrade proration must be accurate");
        
        // Test credit for plan downgrade
        let downgrade_credit = calculate_plan_downgrade_credit(
            new_plan_monthly,
            old_plan_monthly,
            change_date,
            &billing_cycle,
        );
        
        let expected_credit = plan_difference * Decimal::from_str("0.5").unwrap();
        assert_eq!(downgrade_credit, expected_credit, "Plan downgrade credit must be accurate");
        
        // Test daily proration precision (mid-day change)
        let mid_day_change = cycle_start + Duration::days(15) + Duration::hours(12);
        let precise_proration = calculate_precise_proration(
            seat_cost_per_month,
            mid_day_change,
            &billing_cycle,
        );
        
        // Should account for partial day (14.5 days remaining out of 30)
        let remaining_fraction = Decimal::from_str("14.5").unwrap() / Decimal::from(30);
        let expected_precise = seat_cost_per_month * remaining_fraction;
        assert_eq!(precise_proration, expected_precise, "Precise daily proration must be accurate");
        
        println!("✅ Proration accuracy validation passed");
    }
    
    /// Test usage tracking and billing calculations
    /// References: docs/bevy/examples/ecs/event.rs (usage event validation)
    #[test]
    fn test_usage_tracking_accuracy() {
        let mut app = create_test_app_with_billing_system();
        
        let org_id = Uuid::new_v4();
        let mut usage_tracker = create_test_usage_tracker(org_id);
        
        // Test API usage tracking
        let api_usage_events = vec![
            UsageEvent::ApiCallMade {
                organization_id: org_id,
                endpoint: "/api/v1/users".to_string(),
                cost_weight: Some(1.0),
            },
            UsageEvent::ApiCallMade {
                organization_id: org_id,
                endpoint: "/api/v1/analytics".to_string(),
                cost_weight: Some(5.0), // Expensive operation
            },
            UsageEvent::ApiCallMade {
                organization_id: org_id,
                endpoint: "/api/v1/search".to_string(),
                cost_weight: Some(2.0),
            },
        ];
        
        // Process usage events
        for event in api_usage_events {
            process_usage_event(&event, &mut usage_tracker);
        }
        
        // Validate usage accumulation
        let monitor = usage_tracker.usage_monitors.get(&org_id).unwrap();
        assert_eq!(monitor.current_period_usage.api_calls, 3, "API call count must be accurate");
        
        // Validate weighted usage calculation
        let total_weighted_usage = calculate_weighted_usage(
            &monitor.current_period_usage,
            UsageCategory::ApiRequests,
        );
        let expected_weighted = 1.0 + 5.0 + 2.0; // Sum of cost weights
        assert_eq!(total_weighted_usage, expected_weighted, "Weighted usage must sum correctly");
        
        // Test storage usage tracking
        let storage_events = vec![
            UsageEvent::StorageUsed {
                organization_id: org_id,
                bytes_used: 1024 * 1024, // 1 MB
                storage_type: StorageType::Files,
            },
            UsageEvent::StorageUsed {
                organization_id: org_id,
                bytes_used: 512 * 1024, // 512 KB
                storage_type: StorageType::Database,
            },
        ];
        
        for event in storage_events {
            process_usage_event(&event, &mut usage_tracker);
        }
        
        let updated_monitor = usage_tracker.usage_monitors.get(&org_id).unwrap();
        let expected_storage = 1024 * 1024 + 512 * 1024; // Total bytes
        assert_eq!(updated_monitor.current_period_usage.storage_bytes, expected_storage, 
            "Storage usage accumulation must be accurate");
        
        // Test usage-based billing calculation
        let usage_pricing = UsagePricingModel::Tiered {
            tiers: vec![
                UsageTier {
                    tier_name: "Base".to_string(),
                    usage_range: UsageRange { min: 0, max: Some(1000) },
                    price_per_unit: Decimal::from_str("0.01").unwrap(),
                    discount: None,
                },
                UsageTier {
                    tier_name: "Volume".to_string(),
                    usage_range: UsageRange { min: 1001, max: None },
                    price_per_unit: Decimal::from_str("0.005").unwrap(),
                    discount: Some(TierDiscount::Percentage(Decimal::from_str("0.10").unwrap())),
                },
            ],
        };
        
        // Test billing for 1500 API calls
        let billing_amount = calculate_usage_billing(1500, &usage_pricing);
        // First 1000 calls: 1000 * $0.01 = $10.00
        // Next 500 calls: 500 * $0.005 * 0.9 (10% discount) = $2.25
        let expected_billing = Decimal::from_str("10.00").unwrap() + Decimal::from_str("2.25").unwrap();
        assert_eq!(billing_amount, expected_billing, "Tiered usage billing must calculate correctly");
        
        println!("✅ Usage tracking accuracy validation passed");
    }
    
    /// Test payment processing security and accuracy
    /// References: docs/bevy/examples/ecs/event.rs (payment event validation)
    #[test]
    fn test_payment_processing_validation() {
        let mut app = create_test_app_with_billing_system();
        
        let org_id = Uuid::new_v4();
        let invoice_id = Uuid::new_v4();
        let payment_id = Uuid::new_v4();
        
        // Create test invoice
        let invoice = Invoice {
            invoice_id,
            invoice_number: "INV-2024-001".to_string(),
            organization_id: org_id,
            billing_period: BillingPeriod {
                start_date: Utc::now() - Duration::days(30),
                end_date: Utc::now(),
                amount_due: Decimal::from_str("299.95").unwrap(),
                currency: "USD".to_string(),
                status: BillingStatus::Pending,
            },
            generated_at: Utc::now(),
            due_date: Utc::now() + Duration::days(30),
            status: InvoiceStatus::Pending,
            line_items: vec![
                InvoiceLineItem {
                    item_id: Uuid::new_v4(),
                    description: "Professional Plan - 10 seats".to_string(),
                    category: InvoiceItemCategory::Subscription,
                    quantity: 10.0,
                    unit_price: Decimal::from_str("15.99").unwrap(),
                    line_total: Decimal::from_str("159.90").unwrap(),
                    usage_period: None,
                    proration: None,
                    charge_id: None,
                },
                InvoiceLineItem {
                    item_id: Uuid::new_v4(),
                    description: "API Usage Overage".to_string(),
                    category: InvoiceItemCategory::Usage,
                    quantity: 5000.0,
                    unit_price: Decimal::from_str("0.01").unwrap(),
                    line_total: Decimal::from_str("50.00").unwrap(),
                    usage_period: Some(DateRange {
                        start: (Utc::now() - Duration::days(30)).date_naive(),
                        end: Utc::now().date_naive(),
                    }),
                    proration: None,
                    charge_id: None,
                },
            ],
            subtotal: Decimal::from_str("209.90").unwrap(),
            taxes: vec![
                TaxLineItem {
                    tax_id: Uuid::new_v4(),
                    tax_type: TaxType::SalesTax,
                    tax_name: "CA Sales Tax".to_string(),
                    tax_rate: Decimal::from_str("0.0875").unwrap(),
                    taxable_amount: Decimal::from_str("209.90").unwrap(),
                    tax_amount: Decimal::from_str("18.37").unwrap(),
                },
                TaxLineItem {
                    tax_id: Uuid::new_v4(),
                    tax_type: TaxType::LocalTax,
                    tax_name: "SF Local Tax".to_string(),
                    tax_rate: Decimal::from_str("0.0125").unwrap(),
                    taxable_amount: Decimal::from_str("209.90").unwrap(),
                    tax_amount: Decimal::from_str("2.62").unwrap(),
                },
            ],
            total_amount: Decimal::from_str("230.89").unwrap(),
            currency: "USD".to_string(),
            payment_terms: PaymentTerms::Net30,
            customization: InvoiceCustomization::default(),
            delivery_info: InvoiceDeliveryInfo::default(),
        };
        
        // Test invoice calculation accuracy
        let calculated_subtotal: Decimal = invoice.line_items.iter()
            .map(|item| item.line_total)
            .sum();
        assert_eq!(calculated_subtotal, invoice.subtotal, "Invoice subtotal must equal sum of line items");
        
        let calculated_tax_total: Decimal = invoice.taxes.iter()
            .map(|tax| tax.tax_amount)
            .sum();
        let expected_total = invoice.subtotal + calculated_tax_total;
        assert_eq!(invoice.total_amount, expected_total, "Invoice total must equal subtotal plus taxes");
        
        // Test individual tax calculations
        let ca_sales_tax = &invoice.taxes[0];
        let expected_ca_tax = invoice.subtotal * ca_sales_tax.tax_rate;
        assert_eq!(ca_sales_tax.tax_amount, expected_ca_tax.round_dp(2), 
            "CA sales tax calculation must be accurate");
        
        let sf_local_tax = &invoice.taxes[1];
        let expected_sf_tax = invoice.subtotal * sf_local_tax.tax_rate;
        assert_eq!(sf_local_tax.tax_amount, expected_sf_tax.round_dp(2),
            "SF local tax calculation must be accurate");
        
        // Test payment method validation
        let credit_card = PaymentMethod {
            method_id: Uuid::new_v4(),
            organization_id: org_id,
            method_type: PaymentMethodType::CreditCard {
                token: "tok_visa_4242".to_string(),
                brand: CardBrand::Visa,
                last_four: "4242".to_string(),
                expiry: CardExpiry { month: 12, year: 2026 },
                billing_address: BillingAddress::default(),
            },
            display_info: PaymentMethodDisplay {
                display_name: "Visa ending in 4242".to_string(),
                is_default: true,
            },
            status: PaymentMethodStatus::Active,
            security_info: PaymentMethodSecurity {
                is_verified: true,
                verification_date: Some(Utc::now()),
                pci_token: Some("tok_visa_4242".to_string()),
            },
            auto_payment: AutoPaymentConfig {
                enabled: true,
                retry_attempts: 3,
                retry_interval_days: 3,
            },
            preferences: PaymentMethodPreferences::default(),
        };
        
        // Validate payment method security
        assert!(credit_card.security_info.is_verified, "Payment method must be verified");
        assert!(credit_card.security_info.pci_token.is_some(), "PCI token must be present");
        
        // Test payment processing
        let payment_request = PaymentRequest {
            payment_id,
            invoice_id,
            amount: invoice.total_amount,
            currency: invoice.currency.clone(),
            payment_method: credit_card,
            description: format!("Payment for invoice {}", invoice.invoice_number),
        };
        
        let payment_result = validate_payment_request(&payment_request);
        assert!(payment_result.is_ok(), "Valid payment request must pass validation");
        
        // Test payment amount validation
        let incorrect_amount_request = PaymentRequest {
            amount: Decimal::from_str("100.00").unwrap(), // Wrong amount
            ..payment_request.clone()
        };
        let incorrect_result = validate_payment_request(&incorrect_amount_request);
        assert!(incorrect_result.is_err(), "Incorrect payment amount must fail validation");
        
        println!("✅ Payment processing validation passed");
    }
    
    /// Test seat management and billing integration
    /// References: docs/bevy/examples/ecs/change_detection.rs (seat allocation validation)
    #[test]
    fn test_seat_management_billing() {
        let mut app = create_test_app_with_billing_system();
        
        let org_id = Uuid::new_v4();
        
        // Create subscription with seat allocation
        let mut subscription = OrganizationSubscription {
            subscription_id: Uuid::new_v4(),
            organization_id: org_id,
            current_plan: SubscriptionPlan::Professional {
                cost_per_seat: Decimal::from_str("15.99").unwrap(),
                minimum_seats: 5,
                features: ProfessionalFeatures::default(),
                usage_quotas: ProfessionalUsageQuotas::default(),
                volume_discounts: vec![],
            },
            plan_history: vec![],
            status: SubscriptionStatus::Active,
            billing_config: BillingConfiguration::default(),
            current_period: BillingPeriod {
                start_date: Utc::now(),
                end_date: Utc::now() + Duration::days(30),
                amount_due: Decimal::from_str("79.95").unwrap(), // 5 seats minimum
                currency: "USD".to_string(),
                status: BillingStatus::Pending,
            },
            next_period: None,
            seat_allocation: SeatAllocation {
                total_seats: 5,
                occupied_seats: 3,
                available_seats: 2,
                reserved_seats: 0,
                usage_history: vec![],
                auto_scaling: Some(AutoScalingConfig {
                    enabled: true,
                    min_seats: 5,
                    max_seats: 50,
                    scale_up_threshold: 0.8, // Scale up when 80% occupied
                    scale_down_threshold: 0.3, // Scale down when below 30%
                    cooldown_period: Duration::hours(24),
                    cost_limits: CostLimits {
                        max_monthly_cost: Some(Decimal::from_str("1000.00").unwrap()),
                        cost_alert_threshold: Some(Decimal::from_str("800.00").unwrap()),
                    },
                }),
                optimization: SeatOptimization::default(),
                cost_allocation: vec![],
            },
            usage_quotas: UsageQuotas::default(),
            billing_history: BillingHistory::default(),
            payment_details: PaymentDetails::default(),
            preferences: SubscriptionPreferences::default(),
        };
        
        // Test auto-scaling trigger (add members to reach 80% threshold)
        let new_member_id = Uuid::new_v4();
        
        // Add member (4/5 seats = 80%)
        subscription.seat_allocation.occupied_seats = 4;
        subscription.seat_allocation.available_seats = 1;
        
        let should_scale_up = should_trigger_auto_scaling(&subscription.seat_allocation, ScalingDirection::Up);
        assert!(should_scale_up, "Auto-scaling should trigger at 80% occupancy");
        
        // Test auto-scaling cost calculation
        let scaling_cost = calculate_auto_scaling_cost(
            &subscription,
            ScalingDirection::Up,
            2, // Add 2 seats
        );
        
        // Prorated cost for remainder of billing period
        let days_remaining = (subscription.current_period.end_date - Utc::now()).num_days() as f64;
        let total_days = 30.0;
        let proration_factor = days_remaining / total_days;
        let expected_scaling_cost = Decimal::from_str("15.99").unwrap() * Decimal::from(2) * Decimal::try_from(proration_factor).unwrap();
        
        assert_eq!(scaling_cost, expected_scaling_cost, "Auto-scaling cost must be prorated correctly");
        
        // Test seat deallocation and cost credit
        subscription.seat_allocation.occupied_seats = 2; // Down to 40%
        subscription.seat_allocation.available_seats = 3;
        
        let should_scale_down = should_trigger_auto_scaling(&subscription.seat_allocation, ScalingDirection::Down);
        assert!(should_scale_down, "Auto-scaling should trigger scale-down at 40% occupancy");
        
        // Test cost limits enforcement
        subscription.seat_allocation.total_seats = 60; // Would exceed max cost
        let cost_check = check_auto_scaling_cost_limits(&subscription, ScalingDirection::Up, 5);
        assert!(!cost_check, "Auto-scaling must respect cost limits");
        
        // Test seat optimization recommendations
        let optimization_analysis = analyze_seat_optimization(&subscription);
        assert!(optimization_analysis.recommendations.len() > 0, "Seat optimization must provide recommendations");
        
        println!("✅ Seat management billing validation passed");
    }
    
    /// Test system performance and financial accuracy under load
    #[test]
    fn test_billing_system_performance() {
        let mut app = create_test_app_with_billing_system();
        
        // Test large-scale billing calculation performance
        let org_count = 1000;
        let subscriptions = create_test_subscriptions(org_count);
        
        let start_time = std::time::Instant::now();
        
        // Calculate billing for all subscriptions
        let mut total_billing = Decimal::ZERO;
        for subscription in &subscriptions {
            let monthly_cost = calculate_subscription_monthly_cost(subscription);
            total_billing += monthly_cost;
        }
        
        let calculation_duration = start_time.elapsed();
        assert!(calculation_duration.as_millis() < 1000,
            "Billing calculation for {} organizations must complete within 1 second (took: {}ms)",
            org_count, calculation_duration.as_millis());
        
        assert!(total_billing > Decimal::ZERO, "Total billing must be positive");
        
        // Test usage tracking performance
        let start_time = std::time::Instant::now();
        let mut usage_tracker = UsageTracker::default();
        
        // Generate high-volume usage events
        for org_index in 0..100 {
            let org_id = Uuid::new_v4();
            
            // 1000 usage events per organization
            for event_index in 0..1000 {
                let usage_event = UsageEvent::ApiCallMade {
                    organization_id: org_id,
                    endpoint: format!("/api/v1/endpoint{}", event_index % 10),
                    cost_weight: Some((event_index as f64 % 5.0) + 1.0),
                };
                
                process_usage_event(&usage_event, &mut usage_tracker);
            }
        }
        
        let usage_duration = start_time.elapsed();
        assert!(usage_duration.as_secs() < 5,
            "Processing 100k usage events must complete within 5 seconds (took: {}s)",
            usage_duration.as_secs());
        
        // Verify usage accuracy after high-volume processing
        assert_eq!(usage_tracker.usage_monitors.len(), 100, "Must track all organizations");
        
        for (org_id, monitor) in &usage_tracker.usage_monitors {
            assert_eq!(monitor.current_period_usage.api_calls, 1000,
                "Each organization must have exactly 1000 API calls recorded");
        }
        
        // Test concurrent billing operations
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let shared_billing_total = Arc::new(Mutex::new(Decimal::ZERO));
        let mut handles = vec![];
        
        for chunk in subscriptions.chunks(100) {
            let chunk_subscriptions = chunk.to_vec();
            let billing_total = Arc::clone(&shared_billing_total);
            
            let handle = thread::spawn(move || {
                let mut chunk_total = Decimal::ZERO;
                for subscription in &chunk_subscriptions {
                    chunk_total += calculate_subscription_monthly_cost(subscription);
                }
                
                let mut total = billing_total.lock().unwrap();
                *total += chunk_total;
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let concurrent_total = *shared_billing_total.lock().unwrap();
        assert_eq!(concurrent_total, total_billing, "Concurrent billing calculation must match sequential");
        
        println!("✅ Billing system performance validation passed");
        println!("   Billing calculation: {}ms for {} organizations", calculation_duration.as_millis(), org_count);
        println!("   Usage processing: {}ms for 100k events", usage_duration.as_millis());
        println!("   Concurrent billing: accurate across threads");
    }
}

/// QA Helper Functions
fn create_test_app_with_billing_system() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(SubscriptionBillingPlugin)
       .init_resource::<SubscriptionBillingSystem>();
    app
}

fn create_test_subscriptions(count: usize) -> Vec<OrganizationSubscription> {
    let mut subscriptions = Vec::new();
    
    for i in 0..count {
        let plan = match i % 3 {
            0 => SubscriptionPlan::Professional {
                cost_per_seat: Decimal::from_str("15.99").unwrap(),
                minimum_seats: 5,
                features: ProfessionalFeatures::default(),
                usage_quotas: ProfessionalUsageQuotas::default(),
                volume_discounts: vec![],
            },
            1 => SubscriptionPlan::Enterprise {
                pricing_structure: EnterprisePricing::PerSeat {
                    cost_per_seat: Decimal::from_str("25.00").unwrap(),
                    minimum_seats: 10,
                },
                features: EnterpriseFeatures::default(),
                usage_quotas: EnterpriseUsageQuotas::default(),
                sla_terms: ServiceLevelAgreements::default(),
                support_tier: SupportTier::Dedicated,
            },
            _ => SubscriptionPlan::Free {
                max_members: 3,
                feature_limits: FeatureLimits::default(),
                usage_quotas: FreeUsageQuotas::default(),
            },
        };
        
        let subscription = OrganizationSubscription {
            subscription_id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            current_plan: plan,
            plan_history: vec![],
            status: SubscriptionStatus::Active,
            billing_config: BillingConfiguration::default(),
            current_period: BillingPeriod {
                start_date: Utc::now(),
                end_date: Utc::now() + Duration::days(30),
                amount_due: Decimal::from_str("99.99").unwrap(),
                currency: "USD".to_string(),
                status: BillingStatus::Pending,
            },
            next_period: None,
            seat_allocation: SeatAllocation::default(),
            usage_quotas: UsageQuotas::default(),
            billing_history: BillingHistory::default(),
            payment_details: PaymentDetails::default(),
            preferences: SubscriptionPreferences::default(),
        };
        
        subscriptions.push(subscription);
    }
    
    subscriptions
}
```

### QA Validation Requirements

#### Financial Accuracy Standards ✅
- [ ] All pricing calculations are mathematically correct to 2 decimal places
- [ ] Proration calculations handle all edge cases (partial days, leap years)
- [ ] Tax calculations comply with regional tax regulations
- [ ] Volume discounts apply correctly at all threshold levels
- [ ] Currency conversion maintains precision across all operations

#### Usage Tracking Standards ✅
- [ ] Usage events are recorded with 100% accuracy
- [ ] Weighted usage calculations aggregate correctly
- [ ] Usage quotas enforce limits without data loss
- [ ] High-volume usage processing maintains performance
- [ ] Usage-based billing calculations are precise

#### Payment Security Standards ✅
- [ ] All payment data is tokenized and PCI-compliant
- [ ] Payment validation catches all error conditions
- [ ] Payment retry logic follows best practices
- [ ] Transaction reconciliation is accurate
- [ ] Payment method validation is comprehensive

#### Performance Standards ✅
- [ ] Billing calculations scale to 1000+ organizations
- [ ] Usage processing handles 100k+ events within time limits
- [ ] Concurrent operations maintain data consistency
- [ ] Memory usage remains bounded during peak operations
- [ ] Database queries are optimized for large datasets

### QA Sign-off Criteria

**VALIDATION PENDING EXECUTION**

All billing system validation tests must pass with zero discrepancies before production deployment approval.

**Required Validation Results:**
1. ✅ Financial accuracy: Zero calculation errors in all billing scenarios
2. ✅ Security compliance: 100% PCI-DSS compliance for payment processing
3. ✅ Performance benchmarks: All operations within specified time limits
4. ✅ Data integrity: No data loss during high-volume processing
5. ✅ Regulatory compliance: Tax calculations accurate for all supported regions

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

**Implementation References:**
- Previous Task 4 implementation for subscription billing system components
- `docs/bevy/examples/time/time.rs:1-200` - Time-based billing cycle validation
- `docs/bevy/examples/ecs/event.rs:1-144` - Event-driven payment and usage validation
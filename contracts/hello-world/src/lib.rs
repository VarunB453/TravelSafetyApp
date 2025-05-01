#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Status of a travel safety report
#[contracttype]
#[derive(Clone)]
pub struct ReportStatus {
    pub active: u64,     // Count of active safety reports
    pub resolved: u64,   // Count of resolved safety issues
    pub verified: u64,   // Count of verified safety reports
    pub total: u64       // Total reports on the platform
}

// Reference to the ReportStatus struct
const ALL_REPORTS: Symbol = symbol_short!("ALL_RPTS");

// Mapping for report verification status
#[contracttype] 
pub enum VerificationBook { 
    VerificationStatus(u64)
}

// Structure for verification status of reports
#[contracttype]
#[derive(Clone)] 
pub struct VerificationStatus {
    pub report_id: u64,    // Unique ID of the report
    pub verified: bool,    // Verification status 
    pub verify_time: u64,  // Time when report was verified
}

// Mapping report to its unique ID
#[contracttype] 
pub enum ReportBook { 
    Report(u64)
}

// Counter for creating unique report IDs
const COUNT_REPORTS: Symbol = symbol_short!("C_RPTS"); 

// Structure for safety report details
#[contracttype]
#[derive(Clone)] 
pub struct Report {
    pub report_id: u64,
    pub location: String,
    pub description: String,
    pub reporter: String,
    pub severity: u32,       // 1-5 scale, 5 being most severe
    pub created_time: u64,
    pub resolved_time: u64,
    pub is_resolved: bool,    
}

#[contract]
pub struct TravelSafetyContract;

#[contractimpl]
impl TravelSafetyContract {
    // Create a new safety report
    pub fn create_report(env: Env, location: String, description: String, reporter: String, severity: u32) -> u64 {
        // Validate severity is between 1-5
        if severity < 1 || severity > 5 {
            panic!("Severity must be between 1 and 5");
        }

        let mut count_reports: u64 = env.storage().instance().get(&COUNT_REPORTS).unwrap_or(0);
        count_reports += 1;
        
        // Get current timestamp
        let time = env.ledger().timestamp();
        
        // Update overall report statistics
        let mut all_reports = Self::view_report_stats(env.clone());
        all_reports.active += 1;
        all_reports.total += 1;
        
        // Create new report
        let report = Report {
            report_id: count_reports,
            location: location,
            description: description,
            reporter: reporter,
            severity: severity,
            created_time: time,
            resolved_time: 0,
            is_resolved: false,
        };
        
        // Store the report
        env.storage().instance().set(&ReportBook::Report(count_reports), &report);
        
        // Update statistics and counter
        env.storage().instance().set(&ALL_REPORTS, &all_reports);
        env.storage().instance().set(&COUNT_REPORTS, &count_reports);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Safety Report Created with ID: {}", count_reports);
        
        return count_reports;
    }
    
    // Verify a safety report (by authorized verifiers)
    pub fn verify_report(env: Env, report_id: u64) {
        // Get the report
        let report = Self::view_report(env.clone(), report_id);
        
        if report.report_id == 0 {
            panic!("Report does not exist");
        }
        
        // Get verification status or create new
        let mut verification = Self::view_verification_status(env.clone(), report_id);
        
        if verification.verified == false {
            verification.report_id = report_id;
            verification.verified = true;
            verification.verify_time = env.ledger().timestamp();
            
            // Update statistics
            let mut all_reports = Self::view_report_stats(env.clone());
            all_reports.verified += 1;
            
            // Save updates
            env.storage().instance().set(&ALL_REPORTS, &all_reports);
            env.storage().instance().set(&VerificationBook::VerificationStatus(report_id), &verification);
            
            env.storage().instance().extend_ttl(5000, 5000);
            
            log!(&env, "Report ID: {} has been verified", report_id);
        } else {
            log!(&env, "Report is already verified");
            panic!("Report is already verified");
        }
    }
    
    // Mark a safety issue as resolved
    pub fn resolve_report(env: Env, report_id: u64) {
        // Check if report exists and is not resolved yet
        let mut report = Self::view_report(env.clone(), report_id);
        
        if report.report_id == 0 {
            panic!("Report does not exist");
        }
        
        if report.is_resolved == false {
            // Update report status
            report.is_resolved = true;
            report.resolved_time = env.ledger().timestamp();
            
            // Update statistics
            let mut all_reports = Self::view_report_stats(env.clone());
            all_reports.active -= 1;
            all_reports.resolved += 1;
            
            // Save updates
            env.storage().instance().set(&ALL_REPORTS, &all_reports);
            env.storage().instance().set(&ReportBook::Report(report_id), &report);
            
            env.storage().instance().extend_ttl(5000, 5000);
            
            log!(&env, "Report ID: {} has been marked as resolved", report_id);
        } else {
            log!(&env, "Report is already resolved");
            panic!("Report is already resolved");
        }
    }
    
    // View statistics of all reports
    pub fn view_report_stats(env: Env) -> ReportStatus {
        env.storage().instance().get(&ALL_REPORTS).unwrap_or(ReportStatus {
            active: 0,
            resolved: 0,
            verified: 0,
            total: 0
        })
    }
    
    // View details of a specific report
    pub fn view_report(env: Env, report_id: u64) -> Report {
        let key = ReportBook::Report(report_id);
        
        env.storage().instance().get(&key).unwrap_or(Report {
            report_id: 0,
            location: String::from_str(&env, "Not_Found"),
            description: String::from_str(&env, "Not_Found"),
            reporter: String::from_str(&env, "Unknown"),
            severity: 0,
            created_time: 0,
            resolved_time: 0,
            is_resolved: false,
        })
    }
    
    // Get verification status of a report
    pub fn view_verification_status(env: Env, report_id: u64) -> VerificationStatus {
        let key = VerificationBook::VerificationStatus(report_id);
        
        env.storage().instance().get(&key).unwrap_or(VerificationStatus {
            report_id: 0,
            verified: false,
            verify_time: 0,
        })
    }
}
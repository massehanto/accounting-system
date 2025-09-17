use rust_decimal::Decimal;

pub struct TaxCalculator;

impl TaxCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_ppn(&self, base_amount: Decimal, rate: Decimal) -> Decimal {
        base_amount * rate / Decimal::new(100, 0)
    }
    
    pub fn calculate_pph21(&self, gross_salary: Decimal, ptkp: Decimal) -> Decimal {
        let taxable_income = (gross_salary - ptkp).max(Decimal::ZERO);
        if taxable_income <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        self.apply_progressive_rates(taxable_income)
    }
    
    pub fn calculate_pph22(&self, import_amount: Decimal, rate: Decimal) -> Decimal {
        import_amount * rate / Decimal::new(100, 0)
    }
    
    pub fn calculate_pph23(&self, service_amount: Decimal, rate: Decimal) -> Decimal {
        service_amount * rate / Decimal::new(100, 0)
    }
    
    pub fn calculate_pph25(&self, monthly_income: Decimal, rate: Decimal) -> Decimal {
        monthly_income * rate / Decimal::new(100, 0)
    }

    pub fn calculate_pph29(&self, annual_income: Decimal, total_pph25_paid: Decimal) -> Decimal {
        let annual_tax = self.apply_progressive_rates(annual_income);
        (annual_tax - total_pph25_paid).max(Decimal::ZERO)
    }

    pub fn calculate_pbb(&self, property_value: Decimal, rate: Decimal) -> Decimal {
        property_value * rate / Decimal::new(100, 0)
    }
    
    fn apply_progressive_rates(&self, income: Decimal) -> Decimal {
        let mut tax = Decimal::ZERO;
        let mut remaining = income;
        
        // 2024 Indonesian tax brackets
        // 5% bracket (0 - 60M)
        if remaining > Decimal::ZERO {
            let bracket_amount = remaining.min(Decimal::new(60_000_000, 0));
            tax += bracket_amount * Decimal::new(5, 0) / Decimal::new(100, 0);
            remaining -= bracket_amount;
        }
        
        // 15% bracket (60M - 250M)
        if remaining > Decimal::ZERO {
            let bracket_amount = remaining.min(Decimal::new(190_000_000, 0));
            tax += bracket_amount * Decimal::new(15, 0) / Decimal::new(100, 0);
            remaining -= bracket_amount;
        }
        
        // 25% bracket (250M - 500M)
        if remaining > Decimal::ZERO {
            let bracket_amount = remaining.min(Decimal::new(250_000_000, 0));
            tax += bracket_amount * Decimal::new(25, 0) / Decimal::new(100, 0);
            remaining -= bracket_amount;
        }
        
        // 30% bracket (above 500M)
        if remaining > Decimal::ZERO {
            tax += remaining * Decimal::new(30, 0) / Decimal::new(100, 0);
        }
        
        tax
    }

    pub fn get_ptkp_amount(&self, marital_status: &str, dependents: u32) -> Decimal {
        match (marital_status, dependents) {
            ("single", 0) => Decimal::new(54_000_000, 0),
            ("married", 0) => Decimal::new(58_500_000, 0),
            ("married", 1) => Decimal::new(63_000_000, 0),
            ("married", 2) => Decimal::new(67_500_000, 0),
            ("married", d) if d >= 3 => Decimal::new(72_000_000, 0),
            _ => Decimal::new(54_000_000, 0), // Default to single
        }
    }

    pub fn validate_npwp(&self, npwp: &str) -> bool {
        // NPWP format: XX.XXX.XXX.X-XXX.XXX (15 digits)
        let clean_npwp: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();
        clean_npwp.len() == 15
    }

    pub fn calculate_tax_penalty(&self, tax_amount: Decimal, days_late: u32) -> Decimal {
        // 2% per month penalty (Indonesian tax law)
        let monthly_penalty_rate = Decimal::new(2, 0) / Decimal::new(100, 0);
        let months_late = Decimal::new(days_late as i64, 0) / Decimal::new(30, 0);
        
        tax_amount * monthly_penalty_rate * months_late
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppn_calculation() {
        let calculator = TaxCalculator::new();
        let base_amount = Decimal::new(1_000_000, 0);
        let rate = Decimal::new(11, 0); // 11% PPN
        
        let ppn = calculator.calculate_ppn(base_amount, rate);
        assert_eq!(ppn, Decimal::new(110_000, 0));
    }

    #[test]
    fn test_pph21_calculation() {
        let calculator = TaxCalculator::new();
        let gross_salary = Decimal::new(10_000_000, 0); // 10M per month
        let ptkp = Decimal::new(54_000_000, 0); // Single PTKP
        
        let annual_salary = gross_salary * Decimal::new(12, 0);
        let pph21 = calculator.calculate_pph21(annual_salary, ptkp);
        
        // Should be positive for salary above PTKP
        assert!(pph21 > Decimal::ZERO);
    }

    #[test]
    fn test_npwp_validation() {
        let calculator = TaxCalculator::new();
        
        assert!(calculator.validate_npwp("01.234.567.8-901.234"));
        assert!(calculator.validate_npwp("012345678901234")); // Without formatting
        assert!(!calculator.validate_npwp("01.234.567.8"));  // Too short
        assert!(!calculator.validate_npwp("invalid"));        // Invalid format
    }
}
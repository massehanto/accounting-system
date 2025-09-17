use chrono::{DateTime, NaiveDate, Utc, Datelike, Duration};

pub struct DateUtils;

impl DateUtils {
    /// Gets the start of fiscal year for Indonesian companies (typically January 1)
    pub fn fiscal_year_start(year: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, 1, 1).unwrap()
    }

    /// Gets the end of fiscal year
    pub fn fiscal_year_end(year: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
    }

    /// Gets the current fiscal year based on Indonesian standard
    pub fn current_fiscal_year() -> i32 {
        let now = Utc::now().date_naive();
        now.year()
    }

    /// Calculates age in years from birth date
    pub fn calculate_age(birth_date: NaiveDate) -> i32 {
        let today = Utc::now().date_naive();
        let mut age = today.year() - birth_date.year();
        
        if today.month() < birth_date.month() || 
           (today.month() == birth_date.month() && today.day() < birth_date.day()) {
            age -= 1;
        }
        
        age
    }

    /// Gets the first day of the month
    pub fn first_day_of_month(date: NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
    }

    /// Gets the last day of the month
    pub fn last_day_of_month(date: NaiveDate) -> NaiveDate {
        let next_month = if date.month() == 12 {
            NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
        };
        next_month - Duration::days(1)
    }

    /// Gets the first day of the quarter
    pub fn first_day_of_quarter(date: NaiveDate) -> NaiveDate {
        let quarter_start_month = match date.month() {
            1..=3 => 1,
            4..=6 => 4,
            7..=9 => 7,
            10..=12 => 10,
            _ => 1,
        };
        NaiveDate::from_ymd_opt(date.year(), quarter_start_month, 1).unwrap()
    }

    /// Gets the last day of the quarter
    pub fn last_day_of_quarter(date: NaiveDate) -> NaiveDate {
        let quarter_end_month = match date.month() {
            1..=3 => 3,
            4..=6 => 6,
            7..=9 => 9,
            10..=12 => 12,
            _ => 3,
        };
        Self::last_day_of_month(
            NaiveDate::from_ymd_opt(date.year(), quarter_end_month, 1).unwrap()
        )
    }

    /// Gets the quarter number (1-4) for a given date
    pub fn get_quarter(date: NaiveDate) -> u32 {
        match date.month() {
            1..=3 => 1,
            4..=6 => 2,
            7..=9 => 3,
            10..=12 => 4,
            _ => 1,
        }
    }

    /// Calculates business days between two dates (excluding weekends)
    pub fn business_days_between(start: NaiveDate, end: NaiveDate) -> i32 {
        let mut current = start;
        let mut count = 0;
        
        while current <= end {
            if current.weekday().number_from_monday() <= 5 {
                count += 1;
            }
            current += Duration::days(1);
        }
        
        count
    }

    /// Adds business days to a date (excluding weekends)
    pub fn add_business_days(date: NaiveDate, days: i32) -> NaiveDate {
        let mut current = date;
        let mut added = 0;
        
        while added < days {
            current += Duration::days(1);
            if current.weekday().number_from_monday() <= 5 {
                added += 1;
            }
        }
        
        current
    }

    /// Checks if a date is a weekend
    pub fn is_weekend(date: NaiveDate) -> bool {
        let weekday = date.weekday().number_from_monday();
        weekday == 6 || weekday == 7 // Saturday or Sunday
    }

    /// Gets Indonesian public holidays for a given year
    pub fn indonesian_public_holidays(year: i32) -> Vec<(NaiveDate, String)> {
        vec![
            (NaiveDate::from_ymd_opt(year, 1, 1).unwrap(), "Tahun Baru".to_string()),
            (NaiveDate::from_ymd_opt(year, 8, 17).unwrap(), "Hari Kemerdekaan".to_string()),
            (NaiveDate::from_ymd_opt(year, 12, 25).unwrap(), "Hari Natal".to_string()),
            // Note: Religious holidays like Idul Fitri, Idul Adha, etc. change yearly
            // and would need to be calculated or looked up from an external source
        ]
    }

    /// Checks if a date is an Indonesian public holiday
    pub fn is_indonesian_public_holiday(date: NaiveDate) -> bool {
        let holidays = Self::indonesian_public_holidays(date.year());
        holidays.iter().any(|(holiday_date, _)| *holiday_date == date)
    }

    /// Gets the date range for a given period
    pub fn get_period_range(period: &str, year: i32, period_number: Option<u32>) -> Option<(NaiveDate, NaiveDate)> {
        match period.to_lowercase().as_str() {
            "year" => Some((
                Self::fiscal_year_start(year),
                Self::fiscal_year_end(year)
            )),
            "quarter" => {
                if let Some(quarter) = period_number {
                    if quarter >= 1 && quarter <= 4 {
                        let start_month = (quarter - 1) * 3 + 1;
                        let start_date = NaiveDate::from_ymd_opt(year, start_month, 1)?;
                        let end_date = Self::last_day_of_quarter(start_date);
                        Some((start_date, end_date))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            "month" => {
                if let Some(month) = period_number {
                    if month >= 1 && month <= 12 {
                        let start_date = NaiveDate::from_ymd_opt(year, month, 1)?;
                        let end_date = Self::last_day_of_month(start_date);
                        Some((start_date, end_date))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

/// Indonesian-specific date utilities
pub struct IndonesianDateUtils;

impl IndonesianDateUtils {
    /// Gets Indonesian month names
    pub fn month_name_indonesian(month: u32) -> &'static str {
        match month {
            1 => "Januari",
            2 => "Februari", 
            3 => "Maret",
            4 => "April",
            5 => "Mei",
            6 => "Juni",
            7 => "Juli",
            8 => "Agustus",
            9 => "September",
            10 => "Oktober",
            11 => "November",
            12 => "Desember",
            _ => "Invalid",
        }
    }

    /// Gets Indonesian day names
    pub fn day_name_indonesian(weekday: u32) -> &'static str {
        match weekday {
            1 => "Senin",
            2 => "Selasa",
            3 => "Rabu", 
            4 => "Kamis",
            5 => "Jumat",
            6 => "Sabtu",
            7 => "Minggu",
            _ => "Invalid",
        }
    }

    /// Gets Indonesian tax reporting periods
    pub fn tax_reporting_periods(year: i32) -> Vec<(String, NaiveDate, NaiveDate)> {
        let mut periods = Vec::new();
        
        // Monthly periods for most taxes
        for month in 1..=12 {
            let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let end = DateUtils::last_day_of_month(start);
            let period_name = format!("Masa Pajak {} {}", 
                Self::month_name_indonesian(month), year);
            periods.push((period_name, start, end));
        }
        
        periods
    }

    /// Gets Indonesian fiscal quarters
    pub fn fiscal_quarters(year: i32) -> Vec<(String, NaiveDate, NaiveDate)> {
        vec![
            (
                format!("Kuartal I {}", year),
                NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(year, 3, 31).unwrap(),
            ),
            (
                format!("Kuartal II {}", year),
                NaiveDate::from_ymd_opt(year, 4, 1).unwrap(),
                NaiveDate::from_ymd_opt(year, 6, 30).unwrap(),
            ),
            (
                format!("Kuartal III {}", year),
                NaiveDate::from_ymd_opt(year, 7, 1).unwrap(),
                NaiveDate::from_ymd_opt(year, 9, 30).unwrap(),
            ),
            (
                format!("Kuartal IV {}", year),
                NaiveDate::from_ymd_opt(year, 10, 1).unwrap(),
                NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
            ),
        ]
    }
}
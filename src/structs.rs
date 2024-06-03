use std::fmt;
use crate::utils::print_in_currency;

// ------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Period {
    pub id: u32,
    pub start_date: String,
    pub end_date: Option<String>,
}

#[derive(Debug)]
pub struct Income {
    pub _id: u32,
    pub label: String,
    pub value: i64, // in cents.
}

#[derive(Debug)]
pub struct Expense {
    pub id: u32,
    pub label: String,
    pub estimate: i64, // in cents.
    pub spent: i64, // in cents.
    pub expense_type: ExpenseType
}

#[derive(Debug)]
pub struct Log {
    pub id: u32,
    pub timer: NaiveDateTime,
    pub action: String,
    pub arg1: Option<String>,
    pub arg2: Option<String>,
    pub arg3: Option<String>
}

#[derive(Debug)]
pub enum ExpenseType {
    FIXED,
    ESTIMATED,
    UNPLANNED
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match& self.end_date {
            Some(d) => {
                write!(f, "Period {}, started on {}, ended on {}.", 
                self.id, 
                self.start_date,
                d)
            },
            None => {
                write!(f, "Period {}, started on {}, ongoing.", 
                self.id, 
                self.start_date)
            }
        }
    }
}

impl fmt::Display for Income {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", 
            self.label, 
            print_in_currency(self.value)
    )}
}

impl fmt::Display for ExpenseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpenseType::FIXED => write!(f, "FIXED"),
            ExpenseType::ESTIMATED => write!(f, "ESTIMATED"),
            ExpenseType::UNPLANNED => write!(f, "UNPLANNED")
        }
    }
}

impl fmt::Display for Expense {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {} spent out of {}", 
            self.label, 
            print_in_currency(self.spent), 
            print_in_currency(self.estimate)
    )}
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bare = get_action_label(&self);

        let arg1 = &(self.arg1.to_owned().unwrap_or("".to_string()));
        let arg2 = &(self.arg2.to_owned().unwrap_or("".to_string()));
        let arg3 = &(self.arg3.to_owned().unwrap_or("".to_string()));

        let res = bare
            .replace("%1", &arg1)
            .replace("%2", &arg2)
            .replace("%3", &arg3);

        write!(f, "{} - {} : {}", 
            self.id.to_string(), 
            self.timer.format("%Y-%m-%d %H:%M:%S").to_string(), 
            res)
    }
}

fn get_action_label(log: &Log) -> &str {
    return match log.action.as_str() {
        "START_PERIOD" => "Started a new period. (#%1)",
        "END_PERIOD" => "Ended period #%1.",
        "ADD_INCOME" => "Added income of %2 : %1.",
        "ADD_EXPENSE" => "Added expense : %1 : estimated %2, spent %3.",
        "UPDATE_ESTIMATE" => "Updated expense %1 : new estimate of %2.",
        "RENAME_ESTIMATE" => "Renamed expense %1 : now labelled %2.",
        "REMOVE_EXPENSE" => "Removed expense %1.",
        "SPEND" => "Spent %2 on %1.",
        "OVERRIDE_SPENDING" => "Set spending of %2 on %1.",
        _ => ""
    }
}

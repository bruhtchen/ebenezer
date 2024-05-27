// ------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------
pub mod structs {
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
}
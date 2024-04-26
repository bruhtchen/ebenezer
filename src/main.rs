use std::env;
use std::fmt;
use rusqlite::{Connection, Result};

// ------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------
#[derive(Debug)]
struct Period {
    id: u32,
    start_date: String,
    end_date: Option<String>,
}

#[derive(Debug)]
struct Income {
    id: u32,
    label: String,
    value: i64, // in cents.
}

#[derive(Debug)]
struct Expense {
    id: u32,
    label: String,
    estimate: i64, // in cents.
    spent: i64, // in cents.
    expense_type: ExpenseType
}

#[derive(Debug)]
enum ExpenseType {
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

// ------------------------------------------------------------
// CORE
// ------------------------------------------------------------
fn main() {
    let conn = init_db().unwrap();

    let period = get_current_period(&conn).unwrap();
    let incomes = get_incomes(&conn, period).unwrap();
    let expenses = get_expenses(&conn, period).unwrap();

    if period == 0 {
        create_period(&conn).expect("Error : cannot initialize the first period !");
    }

    // Act on user input
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => show_balance(&incomes, &expenses),
        2 => {
            let action = &args[1];

            match action.as_str() {
                "--list" => list(&incomes, &expenses),
                "--roll" => {
                    end_period(&conn, period).expect("Error : cannot set an end date for the current period !");
                    create_period(&conn).expect("Error : cannot initialize a new period !");
                },
                "--period" => print!("{}", get_period(&conn, period).expect("Error : cannot find infos on the current period !")),
                _ => panic!("Unknown option !")
            }
        },
        3 => {
            let action = &args[1];

            match action.as_str() {
                "--remove" => {
                    let label = &args[2];
                    let expense = find_expense_by_label(&expenses, &label).expect("Error : no expense found, unable to remove it.");
                    remove_expense(&conn, &expense).expect("Error : unable to remove expense.");
                },
                "--spend" => {
                    let label = &args[2];
                    let expense = find_expense_by_label(&expenses, &label).expect("Error : no expense found, unable to remove it.");
                    spend_all(&conn, &expense).expect("Error : unable to spend all on expense.");
                },
                _ => {
                    // ebenezer [spent] [expense_label]
                    let spent = parse_into_cents(&args[1]);
                    let label = &args[2];
                    let opt_expense = find_expense_by_label(&expenses, &label);

                    match opt_expense {
                        Some(exp) => increment_spending(&conn, &exp, spent).expect("Error : Unable to save the spending !"),
                        None => create_expense(&conn, 1, &label, ExpenseType::UNPLANNED, spent, spent).expect("Error : Unable to create a new expense !"),
                    }
                }    
            }            
        },
        4 => {
            let action = &args[1];

            match action.as_str() {
                "--fixed" => {
                    let value = &args[2];
                    let label = &args[3];

                    let opt_expense = find_expense_by_label(&expenses, &label);

                    match opt_expense {
                        Some(exp) => override_estimate(&conn, &exp, parse_into_cents(value)).expect("Error : Unable to update an expense !"),
                        None => create_expense(&conn, 1, &label, ExpenseType::FIXED, parse_into_cents(value), 0).expect("Error : Unable to create a new expense !"),
                    }
                },
                "--estimate" => {
                    let value = &args[2];
                    let label = &args[3];

                    let opt_expense = find_expense_by_label(&expenses, &label);

                    match opt_expense {
                        Some(exp) => override_estimate(&conn, &exp, parse_into_cents(value)).expect("Error : Unable to update an expense !"),
                        None => create_expense(&conn, 1, &label, ExpenseType::ESTIMATED, parse_into_cents(value), 0).expect("Error : Unable to create a new expense !"),
                    }
                },
                "--income" => {
                    let value = &args[2];
                    let label = &args[3];
                    create_income(&conn, 1, &label,  parse_into_cents(value)).expect("Error : Unable to create a new income !")
                },
                _ => panic!("Unknown option !")
            }
        },
        _ => panic!("Unknown option !")
    }
}

/// Print a detailed account.
fn list(incomes: &Vec<Income>, expenses: &Vec<Expense>) {
    print_list("INCOME", &incomes);
    list_expenses(&expenses);
}

/// List every expense.
fn list_expenses(source: &Vec<Expense>) {
    let mut estimated: Vec<&Expense> = Vec::new();
    let mut fixed: Vec<&Expense> = Vec::new();
    let mut unplanned: Vec<&Expense> = Vec::new();

    for expense in source {
        match expense.expense_type {
            ExpenseType::ESTIMATED => estimated.push(expense),
            ExpenseType::FIXED => fixed.push(expense),
            ExpenseType::UNPLANNED => unplanned.push(expense)
        }
    }

    print_list("FIXED MONTHLY EXPENSES", &fixed);
    print_list("VARIABLE MONTHLY EXPENSES", &estimated);
    print_list("UNPLANNED MONTHLY EXPENSES", &unplanned);
}

fn find_expense_by_label<'a>(expenses: &'a Vec<Expense>, label: &'a str) -> Option<&'a Expense> {
    return expenses.iter().find(|exp| exp.label == label);
}

fn print_list<T: fmt::Display>(label: &str, source: &Vec<T>) {
    println!("--------------- {} ---------------", label);
    
    for item in source {
        println!("{}", item);
    }
}

/// Print the current balance.
fn show_balance(incomes: &Vec<Income>, expenses: &Vec<Expense>) {
    println!("Current balance : {}", print_in_currency(get_balance(incomes, expenses)));
    println!("Estimated balance at end of period : {}", print_in_currency(get_end_of_period_estimate(incomes, expenses)));
}

/// Get the current balance (all incomes, minus all expenses spent)
fn get_balance(incomes: &Vec<Income>, expenses: &Vec<Expense>) -> i64 {
    let income_sum: i64 = incomes.iter()
                        .map(|x| x.value)
                        .sum(); 
                           
    let expense_sum: i64 = expenses.iter()
                        .map(|x| x.spent)
                        .sum();

    return income_sum - expense_sum;
}

/// Get the current balance (all incomes, minus all expenses estimated)
fn get_end_of_period_estimate(incomes: &Vec<Income>, expenses: &Vec<Expense>) -> i64 {
    let income_sum: i64 = incomes.iter()
                        .map(|x| x.value)
                        .sum(); 
                           
    let expense_sum: i64 = expenses.iter()
                        .map(|x| x.estimate)
                        .sum();

    return income_sum - expense_sum;
}

/// Increase the amount spent on an expense line to match the estimate.
fn spend_all(conn: &Connection, expense: &Expense) -> Result<()> {
    return override_spending(&conn, &expense, expense.estimate);
}

// ------------------------------------------------------------
// DATABASE
// ------------------------------------------------------------
fn init_db() -> Result<Connection> {
    let conn = Connection::open("./ebenezer.db3")?;

    conn.execute(
        "create table if not exists periods (
            id integer primary key AUTOINCREMENT,
            start_date date not null,
            end_date date
        )",
        (),
    )?;

    conn.execute(
        "create table if not exists incomes (
            id integer primary key AUTOINCREMENT,
            period_id integer not null references periods(id),
            label text not null,
            value BIGINT not null
        )",
        (),
    )?;

    conn.execute(
        "create table if not exists expenses (
            id integer primary key AUTOINCREMENT,
            period_id integer not null references periods(id),
            label text not null,
            type text not null,
            estimate BIGINT not null,
            spent BIGINT not null
        )",
        (),
    )?;

    Ok(conn)
}

/// Create a new income
fn create_income(conn: &Connection, period: u32, label: &str, value: i64) -> Result<()> {    
    conn.execute(
        "INSERT INTO incomes (period_id, label, value) values (?1, ?2, ?3)",
        (period, label, value),
    )?;

    println!("Saved : New income line {} !", label);
    Ok(())
}

/// Create a new expense
fn create_expense(conn: &Connection, period: u32, label: &str, expense_type: ExpenseType, estimate: i64, spent: i64) -> Result<()> {    
    conn.execute(
        "INSERT INTO expenses (period_id, label, type, estimate, spent) values (?1, ?2, ?3, ?4, ?5)",
        (period, label, expense_type.to_string(), estimate, spent),
    )?;

    println!("Saved : New expense line {} !", label);
    Ok(())
}

/// Set a new estimate for an expense line.
fn override_estimate(conn: &Connection, expense: &Expense, new_estimate: i64) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET estimate = ?1 WHERE id = ?2",
        (new_estimate, expense.id),
    )?;

    println!("Saved : Updated expense line {} !", expense.label);
    Ok(())
}

/// Removes an expense
fn remove_expense(conn: &Connection, expense: &Expense) -> Result<()> {    
    conn.execute(
        "DELETE FROM expenses WHERE id = ?",
        [expense.id],
    )?;

    println!("Removed expense line {} !", expense.label);
    Ok(())
}

fn get_current_period(conn: &Connection) -> Result<u32> {
    let mut stmt = conn.prepare(
        "SELECT ifnull(max(p.id), 0) FROM periods p"
    )?;

    let mut rows = stmt.query([])?;
    let mut res = 0;

    if let Some(row) = rows.next()? {
        res = row.get(0)?;
    }

    Ok(res)
}

/// Set an end date for a period
fn end_period(conn: &Connection, id: u32) -> Result<()> {    
    conn.execute(
        "UPDATE periods SET end_date = DATE('now') WHERE id = ?",
        [id],
    )?;

    Ok(())
}

/// Create a new period
fn create_period(conn: &Connection) -> Result<()> {    
    conn.execute(
        "INSERT INTO periods (start_date) values (DATE('now'))",
        (),
    )?;

    Ok(())
}

/// Get all info on a period
fn get_period(conn: &Connection, id: u32) -> Result<Period> {
    let mut stmt = conn.prepare(
        "SELECT id, start_date, end_date FROM periods p WHERE p.id = ?"
    )?;

    return stmt.query_row([id], |row| {
        Ok(Period {
            id: row.get(0)?,
            start_date: row.get(1)?,
            end_date: row.get(2)?,
        })
    });
}

/// Get all saved incomes
fn get_incomes(conn: &Connection, period: u32) -> Result<Vec<Income>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.label, i.value FROM incomes i WHERE i.period_id = ?"
    )?;

    let incomes_iter = stmt.query_map([period], |row| {
        Ok(Income {
            id: row.get(0)?,
            label: row.get(1)?,
            value: row.get(2)?,
        })
    })?;

    let mut incomes: Vec<Income> = Vec::new();

    for elem in incomes_iter {
        incomes.push(elem.unwrap());
    }

    return Ok(incomes);
}

/// Get all saved expenses
fn get_expenses(conn: &Connection, period: u32) -> Result<Vec<Expense>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.label, e.type, e.estimate, e.spent FROM expenses e WHERE e.period_id = ?"
    )?;

    let expenses_iter = stmt.query_map([period], |row| {
        let raw_expense_type = row.get::<_, String>(2)?;

        let expense_type = match raw_expense_type.as_str() {
            "FIXED" => ExpenseType::FIXED,
            "ESTIMATED" => ExpenseType::ESTIMATED,
            "UNPLANNED" => ExpenseType::UNPLANNED,
            _ => panic!("Unknown expense type !")
        };

        Ok(Expense {
            id: row.get(0)?,
            label: row.get(1)?,
            estimate: row.get(3)?, // in cents.
            spent: row.get(4)?, // in cents.
            expense_type: expense_type
        })
    })?;

    let mut expenses: Vec<Expense> = Vec::new();

    for elem in expenses_iter {
        expenses.push(elem.unwrap());
    }

    return Ok(expenses);
}

/// Increase the amount spent on an expense line.
fn increment_spending(conn: &Connection, expense: &Expense, amount: i64) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET spent = spent + ?1 WHERE id = ?2",
        (amount, expense.id),
    )?;

    println!("Saved : {} spent on {} !", print_in_currency(amount), expense.label);
    Ok(())
}

/// Increase the amount spent on an expense line.
fn override_spending(conn: &Connection, expense: &Expense, new_amount: i64) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET spent = ?1 WHERE id = ?2",
        (new_amount, expense.id),
    )?;

    println!("Override : {} spent on {} !", print_in_currency(new_amount), expense.label);
    Ok(())
}

// ------------------------------------------------------------
// UTILS
// ------------------------------------------------------------
/// Parse a string (hopefully numerical) into an amount in cents.
fn parse_into_cents(value: &str) -> i64 {
    let splitted: Vec<&str> = value.split(&['.', ','][..]).collect();

    if splitted.len() > 2 {
        panic!("Amounts shouldn't have more than one decimal separator. ({})", value);
    }

    let main:i64 = splitted[0].parse()
        .expect("Input isn't a valid amount !");

    let str_cents = splitted.get(1).or(Some(&"0"))
        .expect("Default value 0 should always be present.");

    if str_cents.len() > 2 {
        panic!("More than two digits after decimal place. ({})", value)
    }

    let padded_cents = format!("{:0<2}", str_cents);

    let cents: i64 = padded_cents.parse().expect("Input isn't a valid amount !");
    return main * 100 + cents;
}

fn print_in_currency(amount: i64) -> String {
    let cents = amount % 100;
    let money = amount / 100;
    return format!("{},{:02}€", money, cents);
}
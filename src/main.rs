use std::io::Error;
use std::{fmt, fs};
use config::Config;
use rusqlite::{Connection, Result};
use clap::{Parser, Subcommand};
use crate::structs::{Log, Period, Income, Expense, ExpenseType};
use crate::utils::{parse_into_cents, print_in_currency};
use homedir::get_my_home;

pub mod structs;
pub mod utils;

#[macro_use]
extern crate lazy_static;

// ------------------------------------------------------------
// CONFIG
// ------------------------------------------------------------
lazy_static!{
    pub static ref CONFIG: Config = Config::builder()
        .add_source(config::Environment::with_prefix("EBENEZER").separator("_"))
        .build()
        .unwrap();
}

// ------------------------------------------------------------
// CLI
// ------------------------------------------------------------

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the current DB path
    Database,

    /// List incomes and expenses
    List { id: Option<u32> },

    /// List every transaction of the current period for auditing purposes
    Logs,

    /// List every transaction for auditing purposes
    LogsAll,

    /// Switch to a new period
    Roll,

    /// Display the current period
    Period,

    /// Remove an expense line
    Remove { label: String },

    /// Spend some money on an expense line. If amount is omitted, the whole expense is spent.
    Spend { label: String, amount: Option<String> },

    /// Create a new constant expense line
    Fixed { label: String, amount: String },
    
    /// Create a new estimated expense line
    Estimate { label: String, amount: String },

    /// Create a new constant expense line
    Income { label: String, amount: String },
    
    /// Change the label of an expense line
    Rename { old: String, new: String },
}

// ------------------------------------------------------------
// CORE
// ------------------------------------------------------------
fn prepare_database_dir() -> Result<(), Error> {
    let path = get_my_home().unwrap().unwrap().as_path()
    .join("ebenezer".to_string());

    if !path.exists() || !path.is_dir() {
        return fs::create_dir(path);
    }

    else {
        return Ok(());
    }
}

/// Returns the path to the SQLite DB file, either from the configuration or a reasonable default.
fn get_dbfile() -> String {
    prepare_database_dir().expect("Unable to create database directory");

    return match CONFIG.get::<String>("dbfile") {
        Ok(r) => r,
        _ => {
           return get_my_home().unwrap().unwrap().as_path()
            .join("ebenezer".to_string())
            .join("ebenezer.db3".to_string())
            .to_str()
            .unwrap_or("./ebenezer.db3")
            .to_string();
        }
    }
}

/// Returns the currency symbol to use, either from the configuration or a reasonable default.
pub fn get_currency() -> String {
    return match CONFIG.get::<String>("currency") {
        Ok(r) => r,
        _ => "€".to_string()
    }
}

fn main() {
    let conn = init_db().unwrap();

    let period = get_current_period(&conn).unwrap();
    let incomes = get_incomes(&conn, period).unwrap();
    let expenses = get_expenses(&conn, period).unwrap();

    if period == 0 {
        create_period(&conn).expect("Error : cannot initialize the first period !");
    }

    let cli = Cli::parse();

    match &cli.command {
        Some(cmd) => {
            match cmd {
                Commands::Database => {
                    println!("{}", get_dbfile())
                },
                Commands::Estimate { label, amount } => {
                    let estimate = parse_into_cents(amount);
                    let opt_expense = find_expense_by_label(&expenses, &label);

                    match opt_expense {
                        Some(exp) => override_estimate(&conn, &exp, estimate).expect("Error : Unable to update an expense !"),
                        None => create_expense(&conn, period, &label, ExpenseType::ESTIMATED, estimate, 0).expect("Error : Unable to create a new expense !"),
                    }
                },

                Commands::Fixed { label, amount } => {
                    let estimate = parse_into_cents(amount);
                    let opt_expense = find_expense_by_label(&expenses, &label);

                    match opt_expense {
                        Some(exp) => override_estimate(&conn, &exp, estimate).expect("Error : Unable to update an expense !"),
                        None => create_expense(&conn, period, &label, ExpenseType::FIXED, estimate, 0).expect("Error : Unable to create a new expense !"),
                    }
                },

                Commands::Income { label, amount } => {
                    let cents = parse_into_cents(amount);
                    create_income(&conn, period, &label, cents).expect("Error : Unable to create a new income !")
                },

                Commands::List { id } => {
                    match id {
                        Some(x) => {
                            let period = get_period(&conn, *x).expect("Unable to find period");
                            
                            println!("PERIOD {} : {} -> {}", 
                                period.id, 
                                period.start_date, 
                                period.end_date.unwrap_or("Current".to_string()));
        
                            let list_incomes = get_incomes(&conn, *x).unwrap();
                            let list_expenses = get_expenses(&conn, *x).unwrap();
                            list(&list_incomes, &list_expenses);
                        },

                        None => { list(&incomes, &expenses) }
                    }
                },
                Commands::Logs => {
                    let logs = get_current_logs(&conn);
                    list_logs(&logs.expect("Error : cannot get logs !"));
                },
                Commands::LogsAll => {
                    let logs = get_all_logs(&conn);
                    list_logs(&logs.expect("Error : cannot get logs !"));
                },
                Commands::Period => {
                    print!("{}", get_period(&conn, period).expect("Error : cannot find infos on the current period !"));
                },
                Commands::Remove { label } => {
                    let expense = find_expense_by_label(&expenses, label.as_str())
                                           .expect("Error : no expense found, unable to remove it.");
                    remove_expense(&conn, &expense).expect("Error : unable to remove expense.");
                },
                Commands::Rename { old, new } => {
                    let opt_expense = find_expense_by_label(&expenses, old.as_str());

                    match opt_expense {
                        Some(exp) => rename_expense(&conn, &exp, new).expect("Error : Unable to update an expense !"),
                        None => panic!("Error : no expense line found"),
                    }
                },
                Commands::Roll => {
                    end_period(&conn, period).expect("Error : cannot set an end date for the current period !");
                    create_period(&conn).expect("Error : cannot initialize a new period !");
                    copy_fixed_and_estimates(&conn).expect("Error : cannot initialize expenses for the new period !");
                },
                Commands::Spend { label, amount } => {
                    match amount {
                        Some(mtt) => {
                            let spent = parse_into_cents(mtt);
                            let opt_expense = find_expense_by_label(&expenses, &label);
        
                            match opt_expense {
                                Some(exp) => increment_spending(&conn, &exp, spent).expect("Error : Unable to save the spending !"),
                                None => create_expense(&conn, period, &label, ExpenseType::UNPLANNED, spent, spent).expect("Error : Unable to create a new expense !"),
                            }
                        },
                        None => {
                            let expense = find_expense_by_label(&expenses, &label).expect("Error : no expense found, unable to remove it.");
                            spend_all(&conn, &expense).expect("Error : unable to spend all on expense.");
                        }
                    }
                },
            }
        },
        None => {
            show_balance(&incomes, &expenses)
        }
    }
}

/// Print the logbook.
fn list_logs(logs: &Vec<Log>) {
    for line in logs {
        println!("{}", line)
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

    estimated.sort_by(|a, b| b.spent.cmp(&a.spent));
    fixed.sort_by(|a, b| b.spent.cmp(&a.spent));
    unplanned.sort_by(|a, b| b.spent.cmp(&a.spent));

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
    let conn = Connection::open(get_dbfile())?;

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
    
    conn.execute(
        "create table if not exists logs (
            id integer primary key AUTOINCREMENT,
            period_id integer not null,
            timer timestamp not null,
            action text not null,
            arg1 text,
            arg2 text,
            arg3 text
        )",
        (),
    )?;

    Ok(conn)
}

/// Create a new line of log with one variable parameter
fn create_log_one_param(conn: &Connection, action: &str, param1: &str) -> Result<()> {    
    let period_id = get_current_period(&conn).expect("Unable to find period !");

    conn.execute(
        "INSERT INTO logs (period_id, timer, action, arg1) values (?1, CURRENT_TIMESTAMP, ?2, ?3)",
        (period_id, action, param1),
    )?;

    Ok(())
}

/// Create a new line of log with two variable parameters
fn create_log_two_params(conn: &Connection, action: &str, param1: &str, param2: &str) -> Result<()> {    
    let period_id = get_current_period(&conn).expect("Unable to find period !");

    conn.execute(
        "INSERT INTO logs (period_id, timer, action, arg1, arg2) values (?1, CURRENT_TIMESTAMP, ?2, ?3, ?4)",
        (period_id, action, param1, param2),
    )?;

    Ok(())
}

/// Create a new line of log with three variable parameters
fn create_log_three_params(conn: &Connection, action: &str, param1: &str, param2: &str, param3: &str) -> Result<()> {    
    let period_id = get_current_period(&conn).expect("Unable to find period !");

    conn.execute(
        "INSERT INTO logs (period_id, timer, action, arg1, arg2, arg3) values (?1, CURRENT_TIMESTAMP, ?2, ?3, ?4, ?5)",
        (period_id, action, param1, param2, param3),
    )?;

    Ok(())
}

/// Create a new income
fn create_income(conn: &Connection, period: u32, label: &str, value: i64) -> Result<()> {    
    conn.execute(
        "INSERT INTO incomes (period_id, label, value) values (?1, ?2, ?3)",
        (period, label, value),
    )?;

    create_log_two_params(&conn, "ADD_INCOME", label, &print_in_currency(value))
        .expect("Unable to create ADD_INCOME log : ");
        
    println!("Saved : New income line {} !", label);
    Ok(())
}

/// Create a new expense
fn create_expense(conn: &Connection, period: u32, label: &str, expense_type: ExpenseType, estimate: i64, spent: i64) -> Result<()> {    
    conn.execute(
        "INSERT INTO expenses (period_id, label, type, estimate, spent) values (?1, ?2, ?3, ?4, ?5)",
        (period, label, expense_type.to_string(), estimate, spent),
    )?;

    create_log_three_params(&conn, "ADD_EXPENSE", &label, &print_in_currency(estimate), &print_in_currency(spent))
        .expect("Unable to create ADD_EXPENSE log : ");
    Ok(())
}

/// Set a new estimate for an expense line.
fn override_estimate(conn: &Connection, expense: &Expense, new_estimate: i64) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET estimate = ?1 WHERE id = ?2",
        (new_estimate, expense.id),
    )?;

    create_log_two_params(&conn, "UPDATE_ESTIMATE", &expense.label, &print_in_currency(new_estimate))
        .expect("Unable to create UPDATE_ESTIMATE log : ");
    Ok(())
}

/// Rename an expense line
fn rename_expense(conn: &Connection, expense: &Expense, new_label: &String) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET label = ?1 WHERE id = ?2",
        (new_label, expense.id),
    )?;

    create_log_two_params(&conn, "RENAME_ESTIMATE", &expense.label, &new_label)
        .expect("Unable to create RENAME_ESTIMATE log : ");
    Ok(())
}

/// Removes an expense
fn remove_expense(conn: &Connection, expense: &Expense) -> Result<()> {    
    conn.execute(
        "DELETE FROM expenses WHERE id = ?",
        [expense.id],
    )?;

    create_log_one_param(&conn, "REMOVE_EXPENSE", &expense.label)
        .expect("Unable to create REMOVE_EXPENSE log : ");
    Ok(())
}

/// Copy fixed and estimated expense lines when we roll over to a new period.
/// The "spent" amount is initialized at zero.
fn copy_fixed_and_estimates(conn: &Connection) -> Result<()> {
    let new_period_id: u32 = get_current_period(conn).expect("Unable to find a period !");
    let old_period_id = new_period_id - 1;
    
    conn.execute(
        "INSERT INTO expenses (period_id, label, type, estimate, spent)
              SELECT period_id + 1, label, type, estimate, 0 
              FROM expenses e2 
              WHERE e2.period_id = ?
              AND e2.type in ('FIXED', 'ESTIMATED') ",
        [old_period_id],
    )?;

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

    create_log_one_param(&conn, "END_PERIOD", &id.to_string())
        .expect("Unable to create END_PERIOD log : ");
    Ok(())
}

/// Create a new period
fn create_period(conn: &Connection) -> Result<()> {    
    conn.execute(
        "INSERT INTO periods (start_date) values (DATE('now'))",
        (),
    )?;

    create_log_one_param(&conn, "START_PERIOD", &conn.last_insert_rowid().to_string())
        .expect("Unable to create START_PERIOD log : ");
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
            _id: row.get(0)?,
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

    create_log_two_params(&conn, "SPEND", &expense.label, &print_in_currency(amount))
        .expect("Unable to create SPEND log : ");
    Ok(())
}

/// Increase the amount spent on an expense line.
fn override_spending(conn: &Connection, expense: &Expense, new_amount: i64) -> Result<()> {
    conn.execute(
        "UPDATE expenses SET spent = ?1 WHERE id = ?2",
        (new_amount, expense.id),
    )?;

    create_log_two_params(&conn, "OVERRIDE_SPENDING", &expense.label, &print_in_currency(new_amount))
        .expect("Unable to create OVERRIDE_SPENDING log : ");
    Ok(())
}

// ------------------------------------------------------------
// LOGS
// ------------------------------------------------------------
fn get_all_logs(conn: &Connection) -> Result<Vec<Log>> {
    let mut stmt = conn.prepare(
        "SELECT l.id, l.timer, l.action, l.arg1, l.arg2, l.arg3 FROM logs l ORDER BY l.id desc "
    )?;

    let log_iter = stmt.query_map([], |row| {
        Ok(Log {
            id: row.get(0)?,
            timer: row.get(1)?,
            action: row.get(2)?, 
            arg1: row.get(3)?,
            arg2: row.get(4)?,
            arg3: row.get(5)?,
        })
    })?;

    let mut logs: Vec<Log> = Vec::new();

    for logline in log_iter {
        logs.push(logline.unwrap());
    }

    return Ok(logs);
}

fn get_current_logs(conn: &Connection) -> Result<Vec<Log>> {
    let period_id = get_current_period(&conn).expect("Unable to find period !");

    let mut stmt = conn.prepare(
        "SELECT l.id, l.timer, l.action, l.arg1, l.arg2, l.arg3 FROM logs l WHERE l.period_id = ? ORDER BY l.id desc "
    )?;

    let log_iter = stmt.query_map([period_id], |row| {
        Ok(Log {
            id: row.get(0)?,
            timer: row.get(1)?,
            action: row.get(2)?, 
            arg1: row.get(3)?,
            arg2: row.get(4)?,
            arg3: row.get(5)?,
        })
    })?;

    let mut logs: Vec<Log> = Vec::new();

    for logline in log_iter {
        logs.push(logline.unwrap());
    }

    return Ok(logs);
}


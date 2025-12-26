use std::fmt::Write;

use litesvm::types::TransactionResult;

pub fn demand_tx_success(tx_result: &TransactionResult) {
    if tx_result.is_ok() {
        return;
    }
    dump_logs(tx_result);
    panic!("Transaction should have succeeded");
}

pub fn demand_tx_failure(tx_result: &TransactionResult) {
    if let Ok(meta) = tx_result {
        panic!("Transaction should have failed, but succeeded: {meta:?}");
    }
}

pub fn demand_logs_contain(expected: &str, tx_result: &TransactionResult) {
    let logs = match tx_result {
        Ok(meta) => &meta.logs,
        Err(meta) => &meta.meta.logs,
    };

    if logs.iter().any(|log| log.contains(expected)) {
        return;
    }

    dump_logs(tx_result);
    panic!("Expected {expected:?} in transaction log");
}

pub fn dump_logs(tx_result: &TransactionResult) {
    let logs = match tx_result {
        Ok(meta) => &meta.logs,
        Err(meta) => &meta.meta.logs,
    };

    let log_output = logs.iter().fold(String::new(), |mut acc, log| {
        writeln!(acc, "> {log}").expect("writing to String cannot fail");
        acc
    });
    println!("Transaction log: {log_output}");
}

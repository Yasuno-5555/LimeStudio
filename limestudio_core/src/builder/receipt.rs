//! Audit Receipt Generation.

pub struct AuditReceipt {
    pub target: String,
    pub signer: String,
    pub method: String,
    pub provenance_cmds: Vec<String>,
    pub timestamp: crate::time::Timestamp,
    pub provenance_hash: String,
}

impl AuditReceipt {
    pub fn get_report(&self) -> String {
        let mut report = "[VERIFIED] LimeBuilder Orchestration Complete.\n".to_string();
        report += "--------------------------------------------------\n";
        report += &format!("- Target:     {}\n", self.target);
        report += &format!("- Signer:     {}\n", self.signer);
        report += &format!("- Method:     {}\n", self.method);
        report += &format!("- Timestamp:  {}\n", self.timestamp);
        report += &format!("- Hash:       {}\n", self.provenance_hash);
        report += "- Provenance:\n";
        for cmd in &self.provenance_cmds {
            report += &format!("    {}\n", cmd);
        }
        report += "--------------------------------------------------\n";
        report
    }
}
